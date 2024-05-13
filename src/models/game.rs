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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum GameState {
    Completed,
    InProgress,
    NoRelease,
    PlayAgain,
    PlaySoon,
    Released,
    Tried,
    Upcoming,
    Other(String)
}

impl GameState {
    // Whether the given state represents a game which has been released. We'll try to update
    // the status of any games we think aren't yet released regularly.
    pub fn is_released(&self) -> bool {
        *self != GameState::NoRelease && *self != GameState::Upcoming
    }
}

impl From<String> for GameState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Completed" => GameState::Completed,
            "In progress" => GameState::InProgress,
            "No release" => GameState::NoRelease,
            "Play again" => GameState::PlayAgain,
            "Play soon" => GameState::PlaySoon,
            "Released" => GameState::Released,
            "Tried" => GameState::Tried,
            "Upcoming" => GameState::Upcoming,
            _ => GameState::Other(s),
        }
    }
}

impl Into<String> for GameState {
    fn into(self) -> String {
        match self {
            GameState::Completed => "Completed".to_string(),
            GameState::InProgress => "In progress".to_string(),
            GameState::NoRelease => "No release".to_string(),
            GameState::PlayAgain => "Play again".to_string(),
            GameState::PlaySoon => "Play soon".to_string(),
            GameState::Released => "Released".to_string(),
            GameState::Tried => "Tried".to_string(),
            GameState::Upcoming => "Upcoming".to_string(),
            GameState::Other(s) => s,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotedGame {
    pub note_id: String,
    pub app_id: Option<GameId>,
    pub state: Option<GameState>,
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

// Represents details of a game newly released and in need of update in notion
#[derive(Debug)]
pub struct ReleasedGame {
    pub note_id: String,
    pub name: String,
}
