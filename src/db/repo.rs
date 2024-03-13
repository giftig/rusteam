use std::collections::{HashMap, HashSet};
use std::num::TryFromIntError;
use std::time::Duration;

use chrono::Utc;
use thiserror::Error;
use tokio_postgres::{Client, Error as PgError};

use crate::models::game::{GameId, NotedGame, PlayedGame};

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
}

pub trait OwnedGamesHandling {
    async fn insert_owned_games(&self, games: &[GameId]) -> Result<()>;
}

pub trait PlayedGamesHandling {
    async fn insert_played_game_updates(&self, updates: &[PlayedGame]) -> Result<u64>;
}

pub trait NotedGamesHandling {
    async fn insert_noted_games(&self, notes: &[NotedGame]) -> Result<()>;
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
            INSERT INTO noted_game (app_id, first_noted, my_rating, notes)
            VALUES($1, $2, $3, $4)
            ON CONFLICT (app_id) DO UPDATE
                SET my_rating = excluded.my_rating,
                    notes = excluded.notes
        "#;

        println!("Inserting {} game notes into noted_game table", notes.len());

        for n in notes {
            self.db
                .execute(
                    q,
                    &[
                        &i64::from(n.id.app_id),
                        &n.first_noted.naive_utc(),
                        &n.my_rating.map(i16::from),
                        &n.notes
                    ]
                )
                .await?;
        }
        Ok(())
    }
}
