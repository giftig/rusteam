pub mod conv;

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use thiserror::Error;
use ureq;

use crate::models::game::{GameDetails, GameId, SteamPlaytime, WishlistedGame};
use crate::models::steam::*;

#[derive(Error, Debug)]
pub enum SteamError {
    #[error("An http error occurred fetching data from steam: {0}")]
    Http(#[from] ureq::Error),
    #[error("An IO error occurred fetching data from steam: {0}")]
    Io(#[from] std::io::Error),
    #[error("A JSON decoding error occurred parsing data from steam: {0}")]
    Json(#[from] serde_json::Error),
    #[error("A data conversion error occurred parsing data from steam: {0}")]
    Conv(String),
}

pub type Result<T> = std::result::Result<T, SteamError>;

pub trait SteamPlayerServiceHandling {
    fn get_owned_games(&self, account_id: &str) -> Result<Vec<GameId>>;
    fn get_played_games(&self, account_id: &str) -> Result<Vec<SteamPlaytime>>;
}

pub trait SteamAppsServiceHandling {
    fn get_all_games(&self) -> Result<Vec<SteamAppIdPair>>;
}

pub trait SteamAppDetailsHandling {
    fn get_game_details(&self, ids: &[GameId]) -> Result<(Vec<GameDetails>, Vec<GameId>)>;
}

pub trait SteamWishlistHandling {
    fn get_wishlist(&self, account_id: &str) -> Result<Vec<WishlistedGame>>;
}


pub trait SteamHandling:
    SteamPlayerServiceHandling +
    SteamAppsServiceHandling +
    SteamAppDetailsHandling +
    SteamWishlistHandling {}

pub struct SteamClient {
    api_key: String,
    api_host: String,  // https://api.steampowered.com
    store_host: String, // https://store.steampowered.com
}

impl SteamClient {
    pub fn new(api_key: &str, api_host: &str, store_host: &str) -> SteamClient {
        SteamClient {
            api_key: api_key.to_string(),
            api_host: api_host.to_string(),
            store_host: store_host.to_string(),
        }
    }

    fn get_owned_games_internal(&self, account_id: &str) -> Result<SteamOwnedGamesResponse> {
        let req = ureq::get(&format!("{}/IPlayerService/GetOwnedGames/v0001/", self.api_host))
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
        let req = ureq::get(&format!("{}/ISteamApps/GetAppList/v2/", &self.api_host));
        let res = req.call()?.into_json::<SteamAllGamesResponse>()?;

        Ok(res.applist.apps)
    }
}

impl SteamClient {
    fn get_game_details_internal(&self, ids: &[GameId]) -> Result<(HashMap<GameId, SteamAppDetails>, Vec<GameId>)> {
        let mut results: HashMap<GameId, SteamAppDetailsResponseEntry> = HashMap::new();
        let mut failures: Vec<GameId> = vec![];

        // Requests have to be made one by one unless we're only getting price_overview
        // TODO: This would be better done with an async http library rather than ureq
        for &id in ids {
            let appid: String = id.into();

            let req = {
                ureq::get(&format!("{}/api/appdetails", &self.store_host))
                    .query("currency", "GBP")
                    .query("appids", &appid)
            };

            let res = {
                match req.call()?.into_json::<SteamAppDetailsResponse>() {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Bad JSON response from steam for appid {}: {}; skipping.", &id, &e);
                        failures.push(id.clone());
                        continue;
                    }
                }
            };

            let details = {
                res
                    .results
                    .into_iter()
                    .map(|(k, v)| (GameId::try_from(k.as_ref()).unwrap(), v))
            };

            results.extend(details);
        }

        Ok((results.into_iter().map(|(k, v)| (k, v.data)).collect(), failures))
    }
}

impl SteamAppDetailsHandling for SteamClient {
    fn get_game_details(&self, ids: &[GameId]) -> Result<(Vec<GameDetails>, Vec<GameId>)> {
        let now = Utc::now();

        let (details, failures) = self.get_game_details_internal(ids)?;
        Ok(
            (
                details
                    .into_iter()
                    .map(|(id, d)| conv::extract_game_details(&id, &d, &now))
                    .collect(),
                failures
            )
        )
    }
}

impl SteamWishlistHandling for SteamClient {
    fn get_wishlist(&self, account_id: &str) -> Result<Vec<WishlistedGame>> {
        let req = ureq::get(&format!("{}/IWishlistService/GetWishlist/v1/", self.api_host))
            .query("key", &self.api_key)
            .query("steamid", &account_id);

        let res = req.call()?.into_json::<SteamWishlistResponse>()?;

        res
            .response
            .items
            .into_iter()
            .map(|item| {
                Ok(
                    WishlistedGame {
                        id: GameId::from(item.appid),
                        wishlisted:
                            Utc
                                .timestamp_opt(item.date_added, 0)
                                .single()
                                .ok_or(
                                    SteamError::Conv(
                                        "Bad unix timestamp for date_added field".to_string()
                                    )
                                )?,
                        deleted: None,
                    }
                )
            })
            .collect()
    }
}

impl SteamHandling for SteamClient {}
