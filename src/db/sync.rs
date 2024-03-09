use std::collections::HashMap;

use chrono::Utc;
use thiserror::Error;

use crate::db::repo::*;
use crate::models::game::PlayedGame;
use crate::steam::*;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("An http error occurred fetching data from steam: {0}")]
    Steam(#[from] SteamError),
    #[error("Error interacting with the database: {0}")]
    Repo(#[from] RepoError),
}

pub type Result<T> = std::result::Result<T, SyncError>;

pub struct Sync {
    steam_account_id: String,
    repo: Repo,
    steam: SteamClient,
}

impl Sync {
    pub fn new(steam_account_id: &str, repo: Repo, steam: SteamClient) -> Sync {
        Sync {
            steam_account_id: steam_account_id.to_string(),
            repo: repo,
            steam: steam
        }
    }

    async fn sync_steam_games(&self) -> Result<()> {
        let all_games: HashMap<u32, String> = self.steam.get_all_games()?
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
