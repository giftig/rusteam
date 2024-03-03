mod db;
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
async fn migrate_db(db_client: &mut PgClient) -> () {
    match migrations::run(&mut db_client).await {
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
    // FIXME: config
    let (mut db_client, conn) = tokio_postgres::connect("host=localhost user=admin password=admin dbname=rusteam", NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            panic!("connection error: {}", e);
        }
    });

    migrate_db(&mut db_client);

    let api_key = env::var("STEAM_API_KEY").unwrap();
    let user_id = env::var("STEAM_USER_ID").unwrap();

    let repo = Repo::new(db_client);
    let steam_client = SteamClient::new(&api_key);
    let sync = Sync::new(&user_id, repo, steam_client);

    sync.sync_steam().await.unwrap();
}
