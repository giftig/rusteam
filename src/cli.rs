mod sync;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "rusteam")]
#[command(version = "0.1.0")]
enum Cli {
    Sync(sync::RunSync),
}

impl Cli {
    async fn run(&self) {
        match self {
            Self::Sync(cmd) => cmd.run().await,
        }
    }
}

pub async fn cli_main() {
    Cli::parse().run().await;
}
