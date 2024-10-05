mod cli;
mod config;
mod db;
mod models;
mod notify;
mod notion;
mod steam;

use tokio;

#[tokio::main]
async fn main() {
    cli::cli_main().await;
}
