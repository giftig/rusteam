use chrono::Utc;
use thiserror::Error;
use ureq;

use crate::models::game::OwnedGame;
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
    fn get_owned_games(&self, account_id: &str) -> Result<Vec<OwnedGame>>;
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

    fn get_owned_games(&self, account_id: &str) -> Result<Vec<OwnedGame>> {
        let req = ureq::get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/")
            .query("key", &self.api_key)
            .query("steamid", &account_id);

        let now = Utc::now();
        let res = req.call()?.into_json::<SteamOwnedGamesResponse>()?;
        Ok(
            res
                .response
                .games
                .into_iter()
                // FIXME: I may need to either rename this field or get the real info from
                // somewhere. For now I'll just use the scrape time as purchased time
                .map(|g| OwnedGame { app_id: g.appid, purchased: now.clone() })
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
