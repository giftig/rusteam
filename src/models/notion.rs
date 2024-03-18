use std::convert::identity;

use chrono::{DateTime, Utc};
use notion::ids::Identifier;
use notion::models::{Page, Properties};
use notion::models::properties::PropertyValue;
use notion::models::text::RichText;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("Bad value found in notion row: {0}")]
    BadValue(String),
    #[error("Missing column in notion row: {0}")]
    MissingColumn(String),
}

pub type Result<T> = std::result::Result<T, ExtractError>;

#[derive(Debug)]
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

impl From<String> for GameState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Completed" => GameState::Completed,
            "InProgress" => GameState::InProgress,
            "NoRelease" => GameState::NoRelease,
            "PlayAgain" => GameState::PlayAgain,
            "PlaySoon" => GameState::PlaySoon,
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
            GameState::InProgress => "InProgress".to_string(),
            GameState::NoRelease => "NoRelease".to_string(),
            GameState::PlayAgain => "PlayAgain".to_string(),
            GameState::PlaySoon => "PlaySoon".to_string(),
            GameState::Released => "Released".to_string(),
            GameState::Tried => "Tried".to_string(),
            GameState::Upcoming => "Upcoming".to_string(),
            GameState::Other(s) => s,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameTag(pub String);

// Read-only model of notes about a game in a Notion database
// Read-only because this would be destructive to write back to Notion:
//   - we simplify rich text
//   - we don't model every field, and don't want to require a code change here to add new fields
#[derive(Debug)]
pub struct GameNote {
    pub id: String,
    pub name: Option<String>,
    pub app_id: Option<String>,
    pub state: Option<GameState>,
    pub tags: Vec<GameTag>,
    pub notes: Option<String>,
    pub rating: Option<u8>,
    pub created_time: DateTime<Utc>
}

// Update a notion page by providing Properties. Properties is a wrapper belonging to the notion
// crate, but it doesn't support updates. It does, however, model notion types and provide serde
// for them. Wrap properties in a request wrapper to allow updating page properties over http(s)
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePage {
    pub properties: Properties,
}

// Turn a Notion richtext field into a plain string by traversing the richtext elements
// and concatenating their plaintext values
fn accumulate_plaintext(value: &[RichText]) -> Option<String> {
    let mut acc: String = "".to_string();
    for rt in value {
        acc.push_str(&rt.plain_text());
    }
    if acc == "".to_owned() { None } else { Some(acc) }
}

fn extract_text(value: &PropertyValue) -> Option<String> {
    match value {
        PropertyValue::Text { rich_text, .. } => accumulate_plaintext(rich_text),
        _ => None
    }
}

fn extract_title(value: &PropertyValue) -> Option<String> {
    match value {
        PropertyValue::Title { title, .. } => accumulate_plaintext(title),
        _ => None
    }
}

fn extract_rating(value: &PropertyValue) -> Option<u8> {
    match value {
        PropertyValue::Number { number, .. } => {
            number.clone().and_then(|n| n.as_u64()).and_then(|n| u8::try_from(n).ok())
        }
        _ => None
    }
}

fn extract_state(value: &PropertyValue) -> Result<Option<GameState>> {
    match value {
        PropertyValue::Select { select, ..} => {
            Ok(select.clone().and_then(|sv| sv.name).map(GameState::from))
        },
        _ => Err(ExtractError::BadValue("Wrong type for State field".to_string())),
    }
}

fn extract_tags(value: &PropertyValue) -> Vec<GameTag> {
    match value {
        PropertyValue::MultiSelect { multi_select, .. } => {
            multi_select
                .clone()
                .unwrap_or(vec![])
                .iter()
                .map::<Option<GameTag>, _>(|tag| Some(GameTag(tag.name.clone()?)))
                .filter_map(identity)
                .collect()
        },
        _ => vec![]
    }
}

fn extract_created_time(value: &PropertyValue) -> Result<DateTime<Utc>> {
    match value {
        PropertyValue::CreatedTime { created_time, .. } => {
            Ok(created_time.clone())
        },
        other => {
            Err(ExtractError::BadValue(format!("Wrong type, expected datetime: {:?}", other)))
        }
    }
}

impl TryFrom<Page> for GameNote {
    type Error = ExtractError;

    fn try_from(page: Page) -> Result<Self> {
        let p = page.properties.properties;

        Ok(
            GameNote {
                id: page.id.value().to_string(),
                name: p.get("Name").and_then(extract_title),
                app_id: p.get("Steam ID").and_then(extract_text),
                state: p.get("State")
                    .map(extract_state)
                    .ok_or(Self::Error::MissingColumn("State".to_string()))
                    .and_then(identity)?,
                tags: p.get("Tags").map(extract_tags).unwrap_or(vec![]),
                notes: p.get("Notes").and_then(extract_text),
                rating: p.get("Rating").and_then(extract_rating),
                created_time: p
                    .get("Created time")
                    .map(extract_created_time)
                    .ok_or(Self::Error::MissingColumn("Created time".to_string()))
                    .and_then(identity)?
            }
        )
    }
}
