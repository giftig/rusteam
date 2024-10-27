mod conv;

use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;

use ::notion::NotionApi;
use ::notion::ids::DatabaseId;
use ::notion::models::Properties;
use ::notion::models::properties::PropertyValue;
use ::notion::models::search::DatabaseQuery;
use thiserror::Error;
use ureq;

use crate::models::game::GameState;
use crate::models::notion::{GameNote, UpdatePage};

#[derive(Debug, Error)]
pub enum NotionError {
    #[error("An error occurred while contacting Notion: {0}")]
    Api(#[from] notion::Error),
    #[error("An http error occurred fetching data from Notion via ureq: {0}")]
    Http(#[from] ureq::Error),
    #[error("Format error: {0}")]
    Fmt(#[from] std::fmt::Error),
}
pub type Result<T> = std::result::Result<T, NotionError>;

#[async_trait]
pub trait NotionHandling {
    async fn get_notes(&self) -> Result<Vec<GameNote>>;
    fn set_game_details(&self, note_id: &str, app_id: &str, name: &str) -> Result<()>;
    fn set_state(&self, note_id: &str, state: &GameState) -> Result<()>;
}

pub struct NotionGamesRepo {
    api: NotionApi,
    database_id: String,
    api_key: String,
    api_host: String,
}

impl NotionGamesRepo {
    pub fn new(api_key: &str, database_id: &str, api_host: &str) -> NotionGamesRepo {
        NotionGamesRepo {
            api: NotionApi::new(api_key.to_string()).unwrap(),
            database_id: database_id.to_string(),
            api_key: api_key.to_string(),
            api_host: api_host.to_string(),
        }
    }

    fn update_row(&self, note_id: &str, props: HashMap<String, PropertyValue>) -> Result<()> {
        let body = UpdatePage { properties: Properties { properties: props } };

        // Notion crate doesn't support this operation so we'll do it directly with ureq
        let url = format!("{}/v1/pages/{}", &self.api_host, note_id);
        ureq::patch(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .set("Notion-Version", "2022-06-28")
            .send_json(&body)?;

        Ok(())
    }
}

#[async_trait]
impl NotionHandling for NotionGamesRepo {
    async fn get_notes(&self) -> Result<Vec<GameNote>> {
        let db_id = DatabaseId::from_str(&self.database_id)?;

        Ok(
            self.api
                .query_database(&db_id, DatabaseQuery {sorts: None, filter: None, paging: None})
                .await?
                .results
                .into_iter()
                .filter_map(|page| match GameNote::try_from(page) {
                    Ok(note) => Some(note),
                    Err(e) => {
                        eprintln!("Skipping unparseable notion row. Error: {:?}", e);
                        None
                    }
                })
                .collect()
        )
    }

    fn set_game_details(&self, note_id: &str, app_id: &str, name: &str) -> Result<()> {
        println!("Setting details in notion for game {}: {}", app_id, name);

        let props: HashMap<String, PropertyValue> = HashMap::from([
            ("Steam ID".to_string(), conv::to_text(app_id)),
            ("Name".to_string(), conv::to_title(name)),
        ]);

        Ok(self.update_row(note_id, props)?)
    }

    fn set_state(&self, note_id: &str, state: &GameState) -> Result<()> {
        let pretty_state: String = state.to_owned().into();
        println!("Setting state in notion: game {} = {}", note_id, &pretty_state);

        let props: HashMap<String, PropertyValue> = HashMap::from([
            ("State".to_string(), conv::to_state(&state))
        ]);

        Ok(self.update_row(note_id, props)?)
    }
}
