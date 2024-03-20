use std::convert::From;
use std::hash::Hash;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GameId {
    pub app_id: u32
}

#[derive(Debug, Error)]
pub enum InvalidGameId {
    #[error("Could not convert string to u32")]
    FromString,
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
impl TryFrom<&str> for GameId {
    type Error = InvalidGameId;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(GameId { app_id: s.parse::<u32>().map_err(|_| Self::Error::FromString)? })
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
    pub note_id: String,
    pub app_id: Option<GameId>,
    pub tags: Vec<String>,
    pub my_rating: Option<u8>,
    pub notes: Option<String>,
    pub first_noted: DateTime<Utc>,
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
    pub id: GameId,
    pub playtime: Duration,
    pub last_played: DateTime<Utc>,
    pub recorded: DateTime<Utc>,
}

// Represents a record in the game_details table
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameDetails {
    pub id: GameId,
    pub description: Option<String>,
    pub controller_support: Option<String>,
    pub coop: bool,
    pub local_coop: bool,
    pub metacritic_percent: Option<u8>,
    pub is_released: bool,
    pub release_date: Option<String>,
    pub recorded: DateTime<Utc>,
}
