use clap::Parser;

use crate::config;
use crate::db;
use crate::db::repo::Repo;
use crate::db::sync::Sync;
use crate::notify::{NotificationHandling, PrintNotifier};
use crate::notion::NotionGamesRepo;
use crate::steam::SteamClient;

#[derive(Debug, Parser)]
pub struct RunSync {}

impl RunSync {
    /// Primary rusteam action: sync data from the official steam API and notion
    pub(super) async fn run(&self) {
        let conf = config::read();

        let mut db_client = db::connect(&conf.db.connection_string()).await;
        db::migrate(&mut db_client).await;

        let repo = Repo::new(db_client);
        let steam_client = SteamClient::new(&conf.steam.api_key);
        let notion = NotionGamesRepo::new(&conf.notion.api_key, &conf.notion.database_id);
        let mut notifier = PrintNotifier::new();

        let mut sync = Sync::new(&conf.steam.user_id, repo, steam_client, notion, &mut notifier);
        sync.sync_steam().await.unwrap();
        sync.sync_notion().await.unwrap();

        notifier.run();
    }
}

