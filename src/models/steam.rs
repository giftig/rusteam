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
