use std::collections::HashMap;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::db::repo::*;
use crate::models::game::{GameId, GameDetails, GameState, NotedGame, PlayedGame};
use crate::models::notion::GameNote;
use crate::notion::{NotionError, NotionHandling};
use crate::steam::*;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Notion error: {0}")]
    Notion(#[from] NotionError),
    #[error("Error interacting with the database: {0}")]
    Repo(#[from] RepoError),
    #[error("An http error occurred fetching data from steam: {0}")]
    Steam(#[from] SteamError),
}

pub enum SyncEvent {
    ReleaseDateUpdated {
        game: GameId,
        prev_text: String,
        prev_date: Option<DateTime<Utc>>,
        new_text: String,
        new_date: Option<DateTime<Utc>>
    },
    Released { game: GameId },
}

pub type Result<T> = std::result::Result<T, SyncError>;

// TODO: Split into SteamSync + NotionSync and abstract over the top for better organisation
pub struct Sync {
    steam_account_id: String,
    // FIXME: Should avoid exposing this, but this may mean Sync shouldn't own it.
    pub repo: Repo,
    steam: Box<dyn SteamHandling>,
    notion: Box<dyn NotionHandling>
}

impl Sync {
    pub fn new(
        steam_account_id: &str,
        repo: Repo,
        steam: Box<dyn SteamHandling>,
        notion: Box<dyn NotionHandling>,
    ) -> Sync {
        Sync { steam_account_id: steam_account_id.to_string(), repo, steam, notion }
    }
}

impl Sync {
    async fn sync_steam_games(&self) -> Result<()> {
        let all_games: HashMap<u32, String> = self.steam
            .get_all_games()?
            .into_iter()
            .map(|game| (game.appid, game.name))
            .collect();

        Ok(self.repo.insert_steam_games(all_games).await?)
    }

    async fn sync_owned_games(&self) -> Result<()> {
        let owned_games = self.steam.get_owned_games(&self.steam_account_id)?;
        self.repo.insert_owned_games(&owned_games).await?;

        Ok(())
    }

    async fn sync_played_games(&self) -> Result<()> {
        let playtime_steam = self.steam.get_played_games(&self.steam_account_id)?;
        let now = Utc::now();

        // Current playtime for all owned games, according to steam
        let played_games: Vec<PlayedGame> = playtime_steam
            .into_iter()
            .map(
                |p| PlayedGame {
                    id: p.id,
                    playtime: p.playtime,
                    last_played: p.last_played,
                    recorded: now.clone(),
                }
            )
            .collect();

        self.repo.insert_played_game_updates(&played_games).await?;

        Ok(())
    }

    /// Check if updated game details entries contain a change to release dates and
    /// notify via the log what these changes were
    /// N.B. These changes will also be written into the release_update table
    async fn check_updated_release_dates(&self, games: &[&GameDetails]) -> Result<Vec<SyncEvent>> {
        println!("Checking for updated release dates...");

        let ids: Vec<&GameId> = games.iter().map(|&g| &g.id).collect();
        let previous_release_dates: HashMap<GameId, String> = self.repo.get_release_dates(&ids).await?;
        let mut updates: Vec<SyncEvent> = vec![];

        for g in games {
            match (previous_release_dates.get(&g.id), &g.release_date) {
                (Some(prev), Some(ref curr)) =>
                    // TODO: Should consider moving the release notification to here instead,
                    // as currently the actual release notifications will only affected noted
                    // games, not wishlisted ones.
                    if prev != curr {
                        let prev_date = conv::parse_release_date(&prev);
                        let new_date = conv::parse_release_date(&curr);

                        self.repo.insert_release_update(&g.id, &prev, &prev_date, &curr, &new_date)
                            .await?;

                        let event = SyncEvent::ReleaseDateUpdated {
                            game: g.id.clone(),
                            prev_text: prev.clone(),
                            prev_date,
                            new_text: curr.clone(),
                            new_date
                        };
                        updates.push(event);
                    }
                _ =>
                    ()
            }
        }

        Ok(updates)
    }

    async fn sync_game_details(&mut self) -> Result<Vec<SyncEvent>> {
        let missing_games = self.repo.get_games_missing_details().await?;
        let noted_games = self.repo.get_upcoming_noted_game_ids().await?;
        let wishlisted_games = self.repo.get_upcoming_wishlisted_game_ids().await?;

        println!(
            "Reading game details from steam. {} missing, upcoming: {} noted, {} wishlisted",
            &missing_games.len(),
            &noted_games.len(),
            &wishlisted_games.len(),
        );

        let refresh_ids: Vec<GameId> = {
            missing_games.into_iter()
                .chain(noted_games.clone().into_iter())
                .chain(wishlisted_games.clone().into_iter())
                .collect()
        };

        let (details, failures) = self.steam.get_game_details(&refresh_ids)?;

        // Collect the retrieved details for noted_games, we'll need them to check release changes
        let mut tracked_details = vec![];
        for d in &details {
            if noted_games.contains(&d.id) || wishlisted_games.contains(&d.id) {
                tracked_details.push(d);
            }
        }

        let events = match self.check_updated_release_dates(&tracked_details).await {
            Ok(evts) => evts,
            Err(e) => {
                eprintln!("Failed to check for updated release dates: {}", e);
                vec![]
            },
        };

        self.repo.insert_game_details(&details).await?;
        self.repo.mark_game_detail_failures(&failures).await;
        Ok(events)
    }

    pub async fn sync_steam(&mut self) -> Result<Vec<SyncEvent>> {
        self.sync_steam_games().await?;
        self.sync_owned_games().await?;
        let events = self.sync_game_details().await?;
        self.sync_played_games().await?;

        Ok(events)
    }
}

impl Sync {
    fn write_app_ids_to_notion(
        &self,
        missing: &[(String, String)],
        found: &HashMap<String, GameId>
    ) -> Result<()> {
        // Add app ids in notion for those we've newly discovered
        for (id, name) in missing {
            if let Some(&app_id) = found.get(name) {
                let cast_app_id: String = app_id.into();
                self.notion.set_game_details(&id, &cast_app_id, &name)?;
            }
        }
        Ok(())
    }

    // Turn GameNote records, representing the data in notion, into NotedGame records, a slightly
    // different view of the data to store in postgres.
    // Fill in app IDs we've derived from postgres if they're missing in the notion data.
    fn derive_noted_games(
        notes: &[GameNote],
        app_ids: &HashMap<String, GameId>
    ) -> Vec<NotedGame> {
        notes
            .iter()
            .filter_map(|n| {
                // Get the app ID from notion if it's already present and can be parsed, and fall
                // back on app_ids found in the db by name otherwise.
                let app_id: Option<GameId> = n
                    .app_id
                    .clone()
                    .and_then(|id| id.as_str().try_into().ok())
                    .or(n.name.clone().and_then(|name| app_ids.get(&name)).copied());

                Some(
                    NotedGame {
                        note_id: n.id.clone(),
                        app_id: app_id,
                        state: n.state.clone(),
                        tags: n.tags.clone().into_iter().map(|t| t.0).collect(),
                        my_rating: n.rating,
                        notes: n.notes.clone(),
                        first_noted: n.created_time.clone(),
                    }
                )
            })
            .collect()
    }

    // Summarise IDs and names of games missing app IDs in notion results
    fn missing_app_ids(notes: &[GameNote]) -> Vec<(String, String)> {
        notes
            .iter()
            .filter_map(|n| {
                if n.app_id.is_none() {
                    n.name.clone().map(|name| (n.id.clone(), name))
                } else {
                    None
                }
            })
            .collect()
    }

    async fn update_release_states(&mut self) -> Result<Vec<SyncEvent>> {
        let updated_games = self.repo.get_newly_released_games().await?;
        let mut events: Vec<SyncEvent> = vec![];

        for record in updated_games {
            // FIXME: Use appid, let notifier resolve name
            events.push(SyncEvent::Released { game: record.game_id.clone() });

            self.notion.set_state(&record.note_id, &GameState::Released)?;
        }
        Ok(events)
    }

    pub async fn sync_notion(&mut self) -> Result<Vec<SyncEvent>> {
        let notes = self.notion.get_notes().await?;

        let missing_app_ids = Self::missing_app_ids(&notes);
        let names: Vec<&str> = missing_app_ids.iter().map(|n| n.1.as_str()).collect();
        let found_app_ids = self.repo.get_appids_by_name(&names).await?;

        let noted_games = Self::derive_noted_games(&notes, &found_app_ids);

        self.repo.insert_noted_games(&noted_games).await?;
        self.write_app_ids_to_notion(&missing_app_ids, &found_app_ids)?;

        // TODO: Populate game tags in postgres
        // Try using fuzzy matching to look up app ids by fuzzy name search
        // N.B. Postgres can do levenshtein directly, just need CREATE EXTENSION IF NOT EXISTS fuzzystrmatch
        Ok(self.update_release_states().await?)
    }
}
