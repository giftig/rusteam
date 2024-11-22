#[cfg(test)]
mod tests;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct SteamOwnedGame {
    pub appid: u32,
    pub playtime_forever: u64,
    pub rtime_last_played: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SteamOwnedGames {
    pub game_count: u32,
    pub games: Vec<SteamOwnedGame>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SteamOwnedGamesResponse {
    pub response: SteamOwnedGames,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SteamAllGamesResponse {
    pub applist: SteamApplist,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SteamApplist {
    pub apps: Vec<SteamAppIdPair>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SteamAppIdPair {
    pub appid: u32,
    pub name: String
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct SteamAppDetailsResponse {
    pub results: HashMap<String, SteamAppDetailsResponseEntry>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SteamAppDetailsResponseEntry {
    pub data: SteamAppDetails,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SteamAppDetails {
    pub short_description: Option<String>,
    pub controller_support: Option<String>,
    pub categories: Vec<Category>,
    pub metacritic: Option<MetacriticScore>,
    pub release_date: Option<ReleaseDate>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Category {
    pub id: u32,
    pub description: String
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ReleaseDate {
    pub coming_soon: bool,
    pub date: String
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct MetacriticScore {
    pub score: u8
}
