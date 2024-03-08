mod db;
pub mod config;
pub mod models;
pub mod steam;

use std::env;
use std::str::FromStr;

use notion::NotionApi;
use notion::ids::DatabaseId;
use notion::models::search::DatabaseQuery;
use tokio;
use tokio_postgres::{Client as DbClient, NoTls};

use crate::db::migrations;
use crate::db::repo::Repo;
use crate::db::sync::Sync;
use crate::steam::SteamClient;

async fn query_notion() -> () {
    let api_key = env::var("NOTION_API_KEY").unwrap();
    let db_id = env::var("NOTION_DATABASE_ID").unwrap();

    let notion = NotionApi::new(api_key).unwrap();
    let res = notion
        .query_database(DatabaseId::from_str(&db_id).unwrap(), DatabaseQuery {sorts: None, filter: None, paging: None})
        .await
        .unwrap()
        .results;

    println!("{:?}", &res);
}

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
    let sync = Sync::new(&conf.steam.user_id, repo, steam_client);

    sync.sync_steam().await.unwrap();
}
