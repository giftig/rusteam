use crate::models::notion::GameNote;

use std::str::FromStr;

use ::notion::NotionApi;
use ::notion::ids::DatabaseId;
use ::notion::models::search::DatabaseQuery;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotionError {
    #[error("An error occurred while contacting Notion: {0}")]
    Api(#[from] notion::Error),
    #[error("Format error: {0}")]
    Fmt(#[from] std::fmt::Error),
}
pub type Result<T> = std::result::Result<T, NotionError>;

pub struct NotionGamesRepo {
    api: NotionApi,
    database_id: String
}

impl NotionGamesRepo {
    pub fn new(api: NotionApi, database_id: &str) -> NotionGamesRepo {
        NotionGamesRepo { api: api, database_id: database_id.to_string() }
    }

    pub async fn get_notes(&self) -> Result<Vec<GameNote>> {
        let db_id = DatabaseId::from_str(&self.database_id)?;

        Ok(
            self.api
                .query_database(&db_id, DatabaseQuery {sorts: None, filter: None, paging: None})
                .await?
                .results
                .into_iter()
                .filter_map(|page| match GameNote::try_from(page.properties) {
                    Ok(note) => Some(note),
                    Err(e) => {
                        eprintln!("Skipping unparseable notion row. Error: {:?}", e);
                        None
                    }
                })
                .collect()
        )
    }
}
