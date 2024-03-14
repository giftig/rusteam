mod db;
pub mod config;
pub mod models;
pub mod notion;
pub mod steam;

use ::notion::NotionApi;
use tokio;
use tokio_postgres::{Client as DbClient, NoTls};

use crate::db::migrations;
use crate::db::repo::Repo;
use crate::db::sync::Sync;
use crate::notion::NotionGamesRepo;
use crate::steam::SteamClient;

// Run migrations or panic
async fn migrate_db(db_client: &mut DbClient) -> () {
    match migrations::run(db_client).await {
        Ok(report) => {
            let count = report.applied_migrations().len();
            println!("Successfully ran {} migrations.", count);
        }
        Err(e) => {
            panic!("Migration error: {}", e);
        }

    }
}

#[tokio::main]
async fn main() {
    let conf = config::read();

    let (mut db_client, conn) = tokio_postgres::connect(&conf.db.connection_string(), NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            panic!("connection error: {}", e);
        }
    });

    migrate_db(&mut db_client).await;

    let repo = Repo::new(db_client);
    let steam_client = SteamClient::new(&conf.steam.api_key);
    let notion = NotionGamesRepo::new(
        NotionApi::new(conf.notion.api_key.clone()).unwrap(),
        &conf.notion.database_id
    );
    let sync = Sync::new(&conf.steam.user_id, repo, steam_client, notion);

//    sync.sync_steam().await.unwrap();
    sync.sync_notion().await.unwrap();
}
