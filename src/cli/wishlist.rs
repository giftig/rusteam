use std::fs;
use std::path::PathBuf;

use clap::Parser;
use thiserror::Error;

use crate::config;
use crate::db;
use crate::db::repo::{Repo, RepoError, WishlistHandling};
use crate::steam::conv;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Could not read from file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to read wishlist data: {0}")]
    Conv(#[from] conv::ConvError),
    #[error("Failed to write wishlist data to db: {0}")]
    Repo(#[from] RepoError),
}

type Result<T> = std::result::Result<T, ImportError>;

#[derive(Debug, Parser)]
pub(super) struct ImportFromFile {
    /// File containing wishlist data fetched from steam's web APIs
    // This can be retrieved manually via browser or curl; we don't try to do it directly here as
    // authentication will be problematic given there's mfa involved and steam doesn't expose
    // these APIs for public use.
    #[arg(short, long)]
    file: String,
    #[arg(short, long)]
    config_file: Option<PathBuf>,
}

impl ImportFromFile {
    pub(super) async fn run(&self) -> Result<()> {
        let conf = config::read(self.config_file.as_ref());
        let mut db_client = db::connect(&conf.db.connection_string()).await;
        db::migrate(&mut db_client).await;

        let repo = Repo::new(db_client);
        let raw = serde_json::from_str(&fs::read_to_string(&self.file)?)?;
        let wishlist = conv::extract_wishlist(&raw)?;

        Ok(repo.update_wishlist(&wishlist).await?)
    }
}
