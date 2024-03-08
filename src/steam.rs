use thiserror::Error;
use ureq;

use crate::models::game::GameId;
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
}

impl SteamPlayerServiceHandling for SteamClient {

    fn get_owned_games(&self, account_id: &str) -> Result<Vec<GameId>> {
        let req = ureq::get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/")
            .query("key", &self.api_key)
            .query("steamid", &account_id);

        let res = req.call()?.into_json::<SteamOwnedGamesResponse>()?;
        Ok(
            res
                .response
                .games
                .into_iter()
                .map(|g| g.appid.into())
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
