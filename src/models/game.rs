use std::convert::From;
use std::hash::Hash;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GameId {
    pub app_id: u32
}

impl From<u32> for GameId {
    fn from(u: u32) -> Self {
        GameId { app_id: u }
    }
}
impl From<i64> for GameId {
    fn from(i: i64) -> Self {
        GameId { app_id: i.try_into().unwrap() }
    }
}
impl Into<String> for GameId {
    fn into(self) -> String {
        self.app_id.to_string()
    }
}
impl Into<i64> for GameId {
    fn into(self) -> i64 {
        self.app_id.into()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotedGame {
    #[serde(flatten)]
    pub id: GameId,
    pub name: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub my_rating: Option<u8>,
    pub release_date: Option<DateTime<Utc>>,
}

// Represents a cleaner / simplified version of SteamOwnedGame to hold playtime details
// TODO: Consider replacing SteamOwnedGame with this model and deserialising directly into it
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SteamPlaytime {
    #[serde(flatten)]
    pub id: GameId,
    pub playtime: Duration,
    pub last_played: DateTime<Utc>,
}

// Represents a record in the played_game table
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlayedGame {
    #[serde(flatten)]
    pub id: GameId,
    pub playtime: Duration,
    pub last_played: DateTime<Utc>,
    pub recorded: DateTime<Utc>,
}
