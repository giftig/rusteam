use std::collections::HashMap;

use chrono::Utc;
use thiserror::Error;

use crate::db::repo::*;
use crate::models::game::{GameId, NotedGame, PlayedGame};
use crate::notion::{NotionError, NotionGamesRepo};
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

pub type Result<T> = std::result::Result<T, SyncError>;

pub struct Sync {
    steam_account_id: String,
    repo: Repo,
    steam: SteamClient,
    notion: NotionGamesRepo
}

impl Sync {
    pub fn new(
        steam_account_id: &str,
        repo: Repo,
        steam: SteamClient,
        notion: NotionGamesRepo
    ) -> Sync {
        Sync {
            steam_account_id: steam_account_id.to_string(),
            repo: repo,
            steam: steam,
            notion: notion
        }
    }

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

    pub async fn sync_steam(&self) -> Result<()> {
        self.sync_steam_games().await?;
        self.sync_owned_games().await?;
        self.sync_played_games().await?;
        Ok(())
    }
}

impl Sync {
    pub async fn sync_notion(&self) -> Result<()> {
        let notes = self.notion.get_notes().await?;

        let names_missing_app_ids: Vec<String> = notes
            .iter()
            .filter_map(|n| if n.app_id.is_none() { n.name.clone() } else { None })
            .collect();

        let app_ids = self.repo.get_appids_by_name(&names_missing_app_ids).await?;

        let noted_games: Vec<NotedGame> = notes
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
                        tags: n.tags.clone().into_iter().map(|t| t.0).collect(),
                        my_rating: n.rating,
                        notes: n.notes.clone(),
                        first_noted: n.created_time.clone(),
                    }
                )
            })
            .collect();

        // TODO: Populate game tags in postgres
        // TODO: Look up app ids by name and insert them back into the notion db
        // TODO: The above, but also use strsim to handle non-exact matches
        // N.B. Postgres can do levenshtein directly, just need CREATE EXTENSION IF NOT EXISTS fuzzystrmatch

        self.repo.insert_noted_games(&noted_games).await?;
        Ok(())
    }
}
