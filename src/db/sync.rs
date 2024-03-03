use std::collections::HashMap;

use thiserror::Error;

use crate::db::repo::{Repo, RepoError, OwnedGamesHandling, SteamGamesHandling};
use crate::steam::{SteamClient, SteamError, SteamAppsServiceHandling, SteamPlayerServiceHandling};

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

    pub async fn sync_steam(&self) -> Result<()> {
        self.sync_steam_games().await?;
        self.sync_owned_games().await?;
        Ok(())
    }
}
