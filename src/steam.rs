use thiserror::Error;
use ureq;

use crate::models::steam::SteamOwnedGamesResponse;

#[derive(Error, Debug)]
pub enum SteamError {
    #[error("An http error occurred fetching data from steam: {0}")]
    Http(#[from] ureq::Error),
    #[error("An IO error occurred fetching data from steam: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SteamError>;

pub trait SteamPlayerServiceHandling {
    fn get_owned_games(&self, account_id: &str) -> Result<SteamOwnedGamesResponse>;
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
    fn get_owned_games(&self, account_id: &str) -> Result<SteamOwnedGamesResponse> {
        let req = ureq::get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/")
            .query("key", &self.api_key)
            .query("steamid", &account_id);

        let res = req.call()?;

        Ok(res.into_json::<SteamOwnedGamesResponse>()?)
    }
}
