use std::time::Duration;

use chrono::{DateTime, Utc};
use thiserror::Error;
use ureq;

use crate::models::game::{GameId, SteamPlaytime};
use crate::models::steam::*;

#[derive(Error, Debug)]
pub enum SteamError {
    #[error("An http error occurred fetching data from steam: {0}")]
    Http(#[from] ureq::Error),
    #[error("An IO error occurred fetching data from steam: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SteamError>;

pub trait SteamPlayerServiceHandling {
    fn get_owned_games(&self, account_id: &str) -> Result<Vec<GameId>>;
    fn get_played_games(&self, account_id: &str) -> Result<Vec<SteamPlaytime>>;
}

pub trait SteamAppsServiceHandling {
    fn get_all_games(&self) -> Result<Vec<SteamAppIdPair>>;
}

pub struct SteamClient {
    api_key: String,
}

impl SteamClient {
    pub fn new(api_key: &str) -> SteamClient {
        SteamClient { api_key: api_key.to_string() }
    }

    fn get_owned_games_internal(&self, account_id: &str) -> Result<SteamOwnedGamesResponse> {
        let req = ureq::get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/")
            .query("key", &self.api_key)
            .query("steamid", &account_id);

        Ok(req.call()?.into_json::<SteamOwnedGamesResponse>()?)
    }
}

impl SteamPlayerServiceHandling for SteamClient {
    fn get_owned_games(&self, account_id: &str) -> Result<Vec<GameId>> {
        Ok(
            self.get_owned_games_internal(account_id)?
                .response
                .games
                .into_iter()
                .map(|g| g.appid.into())
                .collect()
        )
    }

    fn get_played_games(&self, account_id: &str) -> Result<Vec<SteamPlaytime>> {
        Ok(
            self.get_owned_games_internal(account_id)?
                .response
                .games
                .into_iter()
                .map(|g| {
                    SteamPlaytime {
                        id: g.appid.into(),
                        playtime: Duration::new(g.playtime_forever * 60, 0),
                        last_played: DateTime::from_timestamp(g.rtime_last_played.try_into().unwrap(), 0).unwrap_or(Utc::now()),
                    }
                })
                .collect()
        )
    }
}

impl SteamAppsServiceHandling for SteamClient {
    fn get_all_games(&self) -> Result<Vec<SteamAppIdPair>> {
        let req = ureq::get("https://api.steampowered.com/ISteamApps/GetAppList/v2/");
        let res = req.call()?.into_json::<SteamAllGamesResponse>()?;

        Ok(res.applist.apps)
    }
}
