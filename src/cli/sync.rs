use clap::Parser;

use std::collections::HashMap;

use crate::config;
use crate::db;
use crate::db::repo::{Repo, SteamGamesHandling};
use crate::db::sync::{Sync, SyncEvent};
use crate::notion::NotionGamesRepo;
use crate::models::game::GameId;
use crate::steam::SteamClient;

#[derive(Debug, Parser)]
pub struct RunSync {}

pub struct SyncReport {
    repo: Repo,
}

/// Report interesting events collected during the sync process
impl SyncReport {
    fn new(repo: Repo) -> SyncReport {
        SyncReport { repo: repo }
    }

    async fn fetch_game_names(&self, events: &[SyncEvent]) -> HashMap<GameId, String> {
        let ids: Vec<GameId> = {
            events.iter()
                .map(|e| {
                    match e {
                        SyncEvent::ReleaseDateUpdated { game, .. } => Some(game.clone()),
                        SyncEvent::Released { game } => Some(game.clone()),
                    }
                })
                .flatten()
                .collect()
        };

        match self.repo.get_game_names_by_id(&ids).await {
            Ok(res) => res,
            Err(e) => {
                println!("Failed to get game names to generate report! Error: {}", &e);
                HashMap::new()
            },
        }
    }

    async fn run(&self, events: &[SyncEvent]) -> () {
        println!("");
        println!("============================================================");
        println!("=== SYNC REPORT                                          ===");
        println!("============================================================");

        if events.is_empty() {
            println!("Nothing significant to report.");
            println!("");
            return;
        }

        let names_by_id = self.fetch_game_names(events).await;

        for e in events {
            // N.B. I don't use Display because this needs to be updated to first resolve app_id
            // by running a query, anyway; Display won't have sufficient context.
            match e {
                SyncEvent::ReleaseDateUpdated { game, prev, updated } => {
                    let name = {
                        names_by_id
                            .get(&game)
                            .map(|s| s.to_string())
                            .unwrap_or(format!("{}", &game))
                    };

                    println!(
                        "🔎 Release date changed for {}: \"{}\" -> \"{}\"", &name, &prev, &updated
                    )
                },
                SyncEvent::Released { game } => {
                    // TODO: DRY
                    let name = {
                        names_by_id
                            .get(&game)
                            .map(|s| s.to_string())
                            .unwrap_or(format!("{}", &game))
                    };
                    println!("🚀 {} is newly released!", &name)
                },
            }
        }

        println!("");
    }
}

impl RunSync {

    /// Primary rusteam action: sync data from the official steam API and notion
    pub(super) async fn run(&self) {
        let conf = config::read();

        let mut db_client = db::connect(&conf.db.connection_string()).await;
        db::migrate(&mut db_client).await;

        let repo = Repo::new(db_client);
        let steam_client = SteamClient::new(&conf.steam.api_key);
        let notion = NotionGamesRepo::new(&conf.notion.api_key, &conf.notion.database_id);

        let mut sync = Sync::new(&conf.steam.user_id, repo, steam_client, notion);

        let mut events = sync.sync_steam().await.unwrap();
        events.extend(sync.sync_notion().await.unwrap());

        let report = SyncReport::new(sync.repo);
        report.run(&events).await;
    }
}

