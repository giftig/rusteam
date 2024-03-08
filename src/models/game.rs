use std::convert::From;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameId {
    pub app_id: u32
}

impl From<u32> for GameId {
    fn from(u: u32) -> Self {
        GameId { app_id: u }
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlayedGame {
    #[serde(flatten)]
    pub id: GameId,
    pub playtime: Duration,
    pub recorded: DateTime<Utc>,
}
