use tokio;
use rusteam::cli;

#[tokio::main]
async fn main() {
    cli::cli_main().await;
}
