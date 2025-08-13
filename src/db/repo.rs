use std::collections::{HashMap, HashSet};
use std::num::TryFromIntError;
use std::time::Duration;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use thiserror::Error;
use tokio_postgres::{Client, Error as PgError};

use crate::models::game::*;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Error connecting to the database: {0}")]
    Postgres(#[from] PgError),
    #[error("Error converting integer types: {0}")]
    IntConversion(#[from] TryFromIntError),
}

pub type Result<T> = std::result::Result<T, RepoError>;

pub struct Repo {
    db: Client
}

// FIXME: Fix inconsistent naming of app_id, appid, id, game_id, etc.
impl Repo {
    pub fn new(db: Client) -> Repo {
        Repo { db: db }
    }

    async fn get_unknown_app_ids(&self, app_ids: &HashSet<u32>) -> Result<HashSet<u32>> {
        let q = "SELECT app_id FROM steam_game WHERE app_id = ANY ($1)";
        let known: HashSet<u32> = self.db
            .query(q, &[&Vec::from_iter(app_ids.iter().map(|&id| i64::from(id)))])
            .await?
            .into_iter()
            .map(|row| row.get::<usize, i64>(0).try_into().unwrap())
            .collect();

        Ok(app_ids - &known)
    }

    async fn get_latest_played_game_updates(&self) -> Result<HashMap<GameId, Duration>> {
        // Need to cast the playtime interval because of lack of INTERVAL support in the client lib
        // https://github.com/sfackler/rust-postgres/issues/60
        // TODO: either migrate the type in postgres or implement ToSql / FromSql for INTERVAL
        let q = r#"
            SELECT DISTINCT ON (app_id)
                app_id, EXTRACT(epoch FROM playtime)::BIGINT
            FROM played_game
            ORDER BY app_id, playtime DESC
        "#;

        Ok(
            self.db
                .query(q, &[])
                .await?
                .into_iter()
                .map(|row| {
                    let id: GameId = GameId::from(row.get::<usize, i64>(0));
                    let playtime: Duration = Duration::from_secs(
                        row.get::<usize, i64>(1).try_into().unwrap()
                    );

                    (id, playtime)
                })
                .collect()
        )
    }
}

pub trait SteamGamesHandling {
    async fn insert_steam_games<T: AsRef<str>>(&self, games: HashMap<u32, T>) -> Result<()>;
    async fn get_game_names_by_id(&self, ids: &[GameId]) -> Result<HashMap<GameId, String>>;
}

pub trait OwnedGamesHandling {
    async fn insert_owned_games(&self, games: &[GameId]) -> Result<()>;
}

pub trait GameDetailsHandling {
    async fn get_games_missing_details(&self) -> Result<Vec<GameId>>;
    async fn insert_game_details(&self, details: &[GameDetails]) -> Result<()>;
    async fn mark_game_detail_failures(&self, games: &[GameId]) -> ();
    async fn get_release_dates(&self, games: &[&GameId]) -> Result<HashMap<GameId, String>>;
}

pub trait PlayedGamesHandling {
    async fn insert_played_game_updates(&self, updates: &[PlayedGame]) -> Result<u64>;
}

pub trait NotedGamesHandling {
    async fn insert_noted_games(&self, notes: &[NotedGame]) -> Result<()>;
    async fn get_appids_by_name<T: AsRef<str>>(&self, names: &[T]) -> Result<HashMap<String, GameId>>;
    async fn get_upcoming_noted_game_ids(&self) -> Result<Vec<GameId>>;
    async fn get_newly_released_games(&self) -> Result<Vec<ReleasedGame>>;
}

pub trait WishlistHandling {
    async fn update_wishlist(&self, items: &[WishlistedGame]) -> Result<()>;
    async fn get_upcoming_wishlisted_game_ids(&self) -> Result<Vec<GameId>>;
}

pub trait ReleaseUpdateHandling {
    async fn insert_release_update(
        &self,
        game: &GameId,
        prev_text: &str,
        prev_date: &Option<DateTime<Utc>>,
        new_text: &str,
        new_date: &Option<DateTime<Utc>>
    ) -> Result<()>;
}

pub trait IgnoredGamesHandling {
    async fn insert_ignored_games(&self, ids: &[GameId]) -> Result<()>;
}

impl SteamGamesHandling for Repo {
    async fn insert_steam_games<T: AsRef<str>>(&self, games: HashMap<u32, T>) -> Result<()> {
        let ids: HashSet<u32> = games.keys().cloned().collect();
        let unknown_ids: HashSet<u32> = self.get_unknown_app_ids(&ids).await?;

        // TODO: Use a transaction here:
        // https://docs.rs/tokio-postgres/latest/tokio_postgres/struct.Transaction.html
        let q = "INSERT INTO steam_game (app_id, name) VALUES ($1, $2)";
        println!("Inserting {} new steam games into steam_game table", unknown_ids.len());

        for id in unknown_ids {
            let name = games.get(&u32::try_from(id).unwrap());

            if let Some(n) = name {
                self.db.execute(q, &[&i64::from(id), &n.as_ref()]).await?;
            }
        }
        Ok(())
    }

    async fn get_game_names_by_id(&self, ids: &[GameId]) -> Result<HashMap<GameId, String>> {
        let q = r#"SELECT app_id, name FROM steam_game WHERE app_id = ANY ($1)"#;

        Ok(
            self.db
                .query(q, &[&ids.iter().map(|&id| Into::<i64>::into(id.clone())).collect::<Vec<_>>()])
                .await?
                .into_iter()
                .map(|row| (GameId::from(row.get::<usize, i64>(0)), row.get(1)))
                .collect()
        )
    }
}

impl GameDetailsHandling for Repo {
    /// Get games which are being tracked and are missing in the game_details table
    /// Tracked games means those owned or wishlisted
    async fn get_games_missing_details(&self) -> Result<Vec<GameId>> {
        // Limit results to 100 to avoid backfilling hundreds of games at once; we can catch up
        // 100 at a time over several syncs this way.
        // This is just a very basic way of avoiding hitting the Steam API rate limit

        // N.B. we also join to game_details_blacklist to avoid repeatedly scraping games for
        // which steam doesn't return a well-formed definition
        let q = r#"
            WITH tracked AS (SELECT app_id FROM owned_game UNION SELECT app_id FROM wishlist)
            SELECT
                tracked.app_id
            FROM
                tracked
                LEFT JOIN game_details details ON tracked.app_id = details.app_id
                LEFT JOIN game_details_blacklist blacklist ON tracked.app_id = blacklist.app_id
            WHERE
                details.app_id IS NULL AND
                (
                    blacklist.failure_count < 5 OR
                    blacklist.failure_count IS NULL
                )
            LIMIT
                100
        "#;

        Ok(
            self.db
                .query(q, &[]).await?
                .into_iter()
                .map(|row| GameId::from(row.get::<usize, i64>(0)))
                .collect()
        )
    }

    async fn insert_game_details(&self, details: &[GameDetails]) -> Result<()> {
        let q = r#"
            INSERT INTO game_details (
                app_id, description, controller_support, coop, local_coop, metacritic_percent,
                is_released, release_date, release_estimate, recorded
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (app_id) DO UPDATE
                SET description = excluded.description,
                    controller_support = excluded.controller_support,
                    coop = excluded.coop,
                    local_coop = excluded.local_coop,
                    metacritic_percent = excluded.metacritic_percent,
                    is_released = excluded.is_released,
                    release_date = COALESCE(excluded.release_date, NULLIF(excluded.release_date, ''), game_details.release_date),
                    release_estimate = COALESCE(excluded.release_estimate, game_details.release_estimate),
                    recorded = excluded.recorded
        "#;

        let mut row_count: u64 = 0;

        println!("Inserting {} game details into game_details table", details.len());
        for d in details {
            let res = self.db
                .execute(
                    q,
                    &[
                        &Into::<i64>::into(d.id.clone()),
                        &d.description,
                        &d.controller_support,
                        &d.coop,
                        &d.local_coop,
                        &d.metacritic_percent.map(i32::from),
                        &d.is_released,
                        &d.release_date,
                        &d.release_estimate.map(|r| r.naive_utc()),
                        &d.recorded.naive_utc(),
                    ],
                )
                .await;

            match res {
                Ok(c) => row_count += c,
                Err(e) => eprintln!("Couldn't insert game details {}: {}", d.id.app_id, e),
            }
        }

        println!("Inserted {} new game details into game_details table", row_count);
        Ok(())
    }

    async fn mark_game_detail_failures(&self, games: &[GameId]) -> () {
        if games.len() == 0 {
            return ();
        }

        eprintln!(
            "Incrementing failure count for game details which could not be parsed: [{}]",
            games.iter().format(", ")
        );

        // Insert 1 failure into the blacklist table or increment the value already present
        let q = r#"
            INSERT INTO game_details_blacklist (app_id, failure_count)
            VALUES ($1, 1)
            ON CONFLICT (app_id) DO UPDATE SET failure_count = game_details_blacklist.failure_count + 1;
        "#;

        for id in games {
            let res = self.db.execute(q, &[&Into::<i64>::into(id.clone())]).await;
            if let Err(err) = res {
                eprintln!("Failed to increment failure count for id {}: {}", &id, &err);
            }
        }
    }

    async fn get_release_dates(&self, app_ids: &[&GameId]) -> Result<HashMap<GameId, String>> {
        let q = r#"SELECT app_id, release_date FROM game_details WHERE app_id = ANY ($1);"#;

        Ok(
            self.db
                .query(q, &[&app_ids.iter().map(|&id| Into::<i64>::into(id.clone())).collect::<Vec<_>>()])
                .await?
                .into_iter()
                .map(|row| (GameId::from(row.get::<usize, i64>(0)), row.get(1)))
                .collect()
        )
    }
}

impl OwnedGamesHandling for Repo {
    async fn insert_owned_games(&self, games: &[GameId]) -> Result<()> {
        let now = Utc::now().naive_utc();
        let q = "INSERT INTO owned_game(app_id, first_recorded) VALUES($1, $2) ON CONFLICT DO NOTHING";

        println!("Inserting owned games into owned_game table");

        let mut row_count: u64 = 0;

        for id in games.into_iter() {
            match self.db.execute(q, &[&Into::<i64>::into(id.clone()), &now]).await {
                Ok(c) => row_count += c,
                Err(e) => eprintln!("Couldn't insert game {}: {}", id.app_id, e),
            }
        }

        println!("Inserted {} new owned games into owned_game table", row_count);
        Ok(())
    }
}

impl PlayedGamesHandling for Repo {
    async fn insert_played_game_updates(&self, updates: &[PlayedGame]) -> Result<u64> {
        let latest_updates = self.get_latest_played_game_updates().await?;
        let q = r#"
            INSERT INTO played_game(app_id, playtime, last_played, recorded)
            VALUES($1, ($2::TEXT || ' secs')::INTERVAL, $3, $4)
        "#;

        let mut update_count: u64 = 0;

        println!("Inserting playtime updates into played_game table");
        for update in updates {
            let should_update = latest_updates
                .get(&update.id)
                .filter(|&u| *u >= update.playtime)
                .map(|_| false)
                .unwrap_or(true);

            if !should_update {
                continue;
            }
            let id: &i64 = &update.id.into();
            let playtime_secs: String = update.playtime.as_secs().to_string();

            update_count += self.db.execute(
                q,
                &[
                    id,
                    &playtime_secs,
                    &update.last_played.naive_utc(),
                    &update.recorded.naive_utc()
                ]
            ).await?;
        }
        println!("Inserted {} recent updates into played_game table", update_count);

        Ok(update_count)
    }
}


impl NotedGamesHandling for Repo {
    async fn insert_noted_games(&self, notes: &[NotedGame]) -> Result<()> {
        let q = r#"
            INSERT INTO noted_game (note_id, app_id, state, first_noted, my_rating, notes)
            VALUES($1, $2, $3, $4, $5, $6)
            ON CONFLICT (note_id) DO UPDATE
                SET app_id = excluded.app_id,
                    state = excluded.state,
                    my_rating = excluded.my_rating,
                    notes = excluded.notes
        "#;

        println!("Inserting {} game notes into noted_game table", notes.len());

        for n in notes {
            self.db
                .execute(
                    q,
                    &[
                        &n.note_id,
                        &n.app_id.map(|id| i64::from(id.app_id)),
                        &n.state.clone().map(|s| Into::<String>::into(s)),
                        &n.first_noted.naive_utc(),
                        &n.my_rating.map(i16::from),
                        &n.notes
                    ]
                )
                .await?;
        }
        Ok(())
    }

    async fn get_appids_by_name<T: AsRef<str>>(&self, names: &[T]) -> Result<HashMap<String, GameId>> {
        let q = "SELECT app_id, name FROM steam_game WHERE name = ANY ($1)";
        let owned_names: Vec<String> = names.iter().map(|s| s.as_ref().to_string()).collect();

        Ok(
            self.db
                .query(q, &[&owned_names]).await?
                .into_iter()
                .map(|row| (row.get(1), GameId::from(row.get::<usize, i64>(0))))
                .collect()
        )
    }

    /// Retrieve upcoming noted games by checking the game_details table for release state
    async fn get_upcoming_noted_game_ids(&self) -> Result<Vec<GameId>> {
        let q = r#"
            SELECT ng.app_id
            FROM noted_game ng LEFT JOIN game_details gd ON ng.app_id = gd.app_id
            WHERE
                gd.is_released = FALSE OR
                gd.release_estimate IS NOT NULL AND gd.release_estimate > NOW()
        "#;

        Ok(
            self.db
                .query(q, &[]).await?
                .into_iter()
                .map(|row| GameId::from(row.get::<usize, i64>(0)))
                .collect()
        )
    }

    async fn get_newly_released_games(&self) -> Result<Vec<ReleasedGame>> {
        let q = r#"
            SELECT
                ng.note_id,
                ng.app_id
            FROM
                noted_game ng
                LEFT JOIN game_details gd ON ng.app_id = gd.app_id
            WHERE
                ng.app_id IS NOT NULL AND
                (ng.state IS NULL OR ng.state IN ('No release', 'Upcoming')) AND
                gd.is_released IS TRUE
        "#;

        Ok(
            self.db
                .query(q, &[]).await?
                .into_iter()
                .map(|row| {
                    ReleasedGame {
                        note_id: row.get(0),
                        game_id: GameId::from(row.get::<usize, i64>(1)),
                    }
                })
                .collect()
        )
    }
}

impl Repo {
    async fn get_wishlisted_ids(&self) -> Result<HashSet<GameId>> {
        let q = r#"SELECT app_id FROM wishlist WHERE deleted IS NULL"#;

        Ok(
            self.db
                .query(q, &[]).await?
                .into_iter()
                .map(|row| GameId::from(row.get::<usize, i64>(0)))
                .collect()
        )
    }

    async fn delete_wishlist_ids(&self, ids: &[&GameId]) -> Result<()> {
        let now = Utc::now().naive_utc();
        let q = r#"UPDATE wishlist SET deleted = $2 WHERE app_id = ANY ($1)"#;

        self.db
            .execute(
                q,
                &[
                    &ids.iter().map(|&id| Into::<i64>::into(id.clone())).collect::<Vec<_>>(),
                    &now,
                ]
            )
            .await?;

        Ok(())
    }

    async fn insert_wishlist_items(&self, items: &[WishlistedGame]) -> Result<()> {
        // If an item was deleted and readded, unmark as deleted but keep original add date
        let q = r#"
            INSERT INTO wishlist (app_id, wishlisted) VALUES ($1, $2)
            ON CONFLICT (app_id) DO UPDATE SET deleted = NULL
        "#;

        for item in items {
            self.db
                .execute(q, &[&Into::<i64>::into(item.id.clone()), &item.wishlisted.naive_utc()])
                .await?;
        }
        Ok(())
    }
}

impl WishlistHandling for Repo {
    /// Sync the wishlist by marking removed items as deleted and then inserting missing items
    // TODO: it'd be best to do this transactionally, see link above
    async fn update_wishlist(&self, items: &[WishlistedGame]) -> Result<()> {
        let existing_ids = self.get_wishlisted_ids().await?;
        let new_ids: HashSet<GameId> = items.iter().map(|item| item.id.clone()).collect();
        let remove_ids: Vec<&GameId> = existing_ids.difference(&new_ids).collect();

        println!("Marking {} wishlist items as deleted...", remove_ids.len());
        if !remove_ids.is_empty() {
            self.delete_wishlist_ids(&remove_ids).await?;
        }

        let new_items: Vec<WishlistedGame> = {
            items.iter().cloned().filter(|item| !existing_ids.contains(&item.id)).collect()
        };

        println!("Inserting {} new wishlist items...", new_items.len());
        if !new_items.is_empty() {
            self.insert_wishlist_items(&new_items).await?;
        }

        Ok(())
    }

    /// Retrieve upcoming wishlisted games by checking the game_details table for release state
    async fn get_upcoming_wishlisted_game_ids(&self) -> Result<Vec<GameId>> {
        let q = r#"
            SELECT w.app_id
            FROM wishlist w LEFT JOIN game_details gd ON w.app_id = gd.app_id
            WHERE
                gd.is_released = FALSE OR
                gd.release_estimate IS NOT NULL AND gd.release_estimate > NOW()
        "#;

        Ok(
            self.db
                .query(q, &[]).await?
                .into_iter()
                .map(|row| GameId::from(row.get::<usize, i64>(0)))
                .collect()
        )
    }
}

impl ReleaseUpdateHandling for Repo {
    async fn insert_release_update(
        &self,
        game: &GameId,
        prev_text: &str,
        prev_date: &Option<DateTime<Utc>>,
        new_text: &str,
        new_date: &Option<DateTime<Utc>>
    ) -> Result<()> {
        let q = r#"
            INSERT INTO release_update (
                app_id, prev_text, new_text, prev_estimate, new_estimate, recorded
            )
            VALUES ($1, $2, $3, $4, $5, $6);
        "#;
        let now = Utc::now().naive_utc();

        self.db
            .execute(
                q,
                &[
                    &Into::<i64>::into(game.clone()),
                    &prev_text.to_string(),
                    &new_text.to_string(),
                    &prev_date.map(|d| d.naive_utc()),
                    &new_date.map(|d| d.naive_utc()),
                    &now,
                ]
            ).await?;
        Ok(())
    }
}

impl IgnoredGamesHandling for Repo {
    async fn insert_ignored_games(&self, ids: &[GameId]) -> Result<()> {
        let q = r#"
            INSERT INTO ignored_game(app_id) VALUES ($1)
            ON CONFLICT DO NOTHING
        "#;

        for id in ids {
            self.db.execute(q, &[&Into::<i64>::into(id.clone())]).await?;
        }
        Ok(())
    }
}
