pub mod models;
pub mod steam;

use std::env;
use std::str::FromStr;

use notion::NotionApi;
use notion::ids::DatabaseId;
use notion::models::search::DatabaseQuery;
use steam::{SteamClient, SteamPlayerServiceHandling};
use tokio;

fn query_steam() -> () {
    let api_key = env::var("STEAM_API_KEY").unwrap();
    let user_id = env::var("STEAM_USER_ID").unwrap();

    let client = SteamClient::new(&api_key);
    let response = client.get_owned_games(&user_id).unwrap();

    let game_count = response.response.game_count;
    let steam_game = response.response.games.first().unwrap();

    println!("Game count: {:?}, first in list: {:?}", game_count, &steam_game);
}

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

#[tokio::main]
async fn main() {
//    query_steam();
//    query_notion().await;
}
