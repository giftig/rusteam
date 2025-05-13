use std::path::PathBuf;

use clap::Parser;

use crate::config;
use crate::db;
use crate::db::repo::{IgnoredGamesHandling, Repo};
use crate::models::game::GameId;

#[derive(Debug, Parser)]
pub struct RunIgnoreGame {
    #[arg(short, long)]
    config_file: Option<PathBuf>,
    #[arg(
      short, long, required = true, num_args = 1.., value_delimiter = ',',
      help = "App IDs to ignore (comma-separated)"
    )]
    games: Vec<String>,
}

impl RunIgnoreGame {
    pub(super) async fn run(&self) {
        let conf = config::read(self.config_file.as_ref());
        let mut db_client = db::connect(&conf.db.connection_string()).await;
        db::migrate(&mut db_client).await;

        let repo = Repo::new(db_client);

        let ids: Vec<GameId> = {
            self.games.clone().into_iter().map(|s| GameId::try_from(s.as_ref()).unwrap()).collect()
        };

        println!("Marking {} games as ignored...", ids.len());
        if let Err(e) = repo.insert_ignored_games(&ids).await {
            panic!("Error while ignoring games: {}", e)
        }
    }
}
