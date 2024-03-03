use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Game {
    pub app_id: u32,
    pub name: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub my_rating: Option<u8>,
    pub release_date: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OwnedGame {
    pub app_id: u32,
    pub purchased: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlayedGame {
    pub app_id: u32,
    pub playtime: Duration,
    pub recorded: DateTime<Utc>,
}
