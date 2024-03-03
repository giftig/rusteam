use refinery::{Report, embed_migrations};
use thiserror::Error;
use tokio_postgres::Client;

embed_migrations!("migrations/");

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Error connecting to the database: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("Error running migrations: {0}")]
    Refinery(#[from] refinery::Error)
}

type Result<T> = std::result::Result<T, MigrationError>;

pub async fn run(client: &mut Client) -> Result<Report> {
    Ok(migrations::runner().run_async(client).await?)
}
