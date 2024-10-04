mod cli;
mod config;
mod db;
mod models;
mod notify;
mod notion;
mod steam;

use tokio;

use crate::cli::run;

#[tokio::main]
async fn main() {
    run().await;
}
