pub mod models;
pub mod steam;

use std::env;

use steam::{SteamClient, SteamPlayerServiceHandling};

fn main() {
    let api_key = env::var("STEAM_API_KEY").unwrap();
    let user_id = env::var("STEAM_USER_ID").unwrap();

    let client = SteamClient::new(&api_key);
    let response = client.get_owned_games(&user_id).unwrap();

    let game_count = response.response.game_count;
    let steam_game = response.response.games.first().unwrap();

    println!("Game count: {:?}, first in list: {:?}", game_count, &steam_game);
}
