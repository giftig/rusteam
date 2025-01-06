use std::collections::HashSet;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use mockall::*;
use mockall::predicate::*;
use tokio;

use rusteam::db;
use rusteam::db::repo::Repo;
use rusteam::db::sync::Sync;
use rusteam::notion::{NotionHandling, Result as NotionResult};
use rusteam::steam::{Result as SteamResult, *};
use rusteam::models::game::{GameDetails, GameId, GameState, SteamPlaytime, WishlistedGame};
use rusteam::models::notion::GameNote;
use rusteam::models::steam::SteamAppIdPair;

mock! {
    pub SteamClient {}

    impl SteamPlayerServiceHandling for SteamClient {
        fn get_owned_games(&self, account_id: &str) -> SteamResult<Vec<GameId>>;
        fn get_played_games(&self, account_id: &str) -> SteamResult<Vec<SteamPlaytime>>;
    }
    impl SteamAppsServiceHandling for SteamClient {
        fn get_all_games(&self) -> SteamResult<Vec<SteamAppIdPair>>;
    }
    impl SteamAppDetailsHandling for SteamClient {
        fn get_game_details(&self, ids: &[GameId]) -> SteamResult<(Vec<GameDetails>, Vec<GameId>)>;
    }
    impl SteamWishlistHandling for SteamClient {
        fn get_wishlist(&self, account_id: &str) -> Result<Vec<WishlistedGame>>;
    }
    impl SteamHandling for SteamClient {}
}

mock! {
    #[async_trait]
    pub NotionClient {}

    #[async_trait]
    impl NotionHandling for NotionClient {
        async fn get_notes(&self) -> NotionResult<Vec<GameNote>>;
        fn set_game_details(&self, note_id: &str, app_id: &str, name: &str) -> NotionResult<()>;
        fn set_state(&self, note_id: &str, state: &GameState) -> NotionResult<()>;
    }
}

fn steam_app_list_fixture() -> Vec<SteamAppIdPair> {
    vec![
        SteamAppIdPair { appid: 666, name: "Game Buying Simulator 2024".to_string() },
        SteamAppIdPair { appid: 1337, name: "Final Fantasy MMLXVII".to_string() },
        SteamAppIdPair { appid: 654321, name: "Paint Drying Tycoon 2".to_string() },
        SteamAppIdPair { appid: 666666, name: "Unearthed Myths 7: The Unearthening".to_string() },
    ]
}

fn steam_owned_games_fixture() -> Vec<GameId> {
    vec![GameId { app_id: 666 }, GameId { app_id: 1337 }]
}

/// N.B. the get_game_details signature returns (results, failures) as some game details might
/// no longer be available in the store
fn steam_game_details_fixture() -> (Vec<GameDetails>, Vec<GameId>) {
    let now = Utc::now();

    (
        vec![
            GameDetails {
                id: GameId { app_id: 666 },
                description: Some("The thrill of buying games I'll never play!".to_string()),
                controller_support: Some("partial".to_string()),
                coop: false,
                local_coop: false,
                metacritic_percent: None,
                is_released: true,
                release_date: Some("1 Jan, 2002".to_string()),
                release_estimate: None,
                recorded: now.clone()
            },
            GameDetails {
                id: GameId { app_id: 1337 },
                description: Some("Now with even better graphics".to_string()),
                controller_support: Some("full".to_string()),
                coop: true,
                local_coop: true,
                metacritic_percent: Some(100),
                is_released: false,
                release_date: Some("Q3 2077".to_string()),
                release_estimate: None,
                recorded: now.clone()
            },
            GameDetails {
                id: GameId { app_id: 666666 },
                description: Some("Unearth some unearthenings".to_string()),
                controller_support: Some("full".to_string()),
                coop: false,
                local_coop: false,
                metacritic_percent: None,
                is_released: false,
                release_date: Some("Coming soon".to_string()),
                release_estimate: None,
                recorded: now.clone()
            },
        ],
        vec![]
    )
}

fn steam_played_games_fixture() -> Vec<SteamPlaytime> {
    vec![
        SteamPlaytime {
            id: GameId { app_id: 666 },
            playtime: Duration::new(60 * 60, 0),  // 1h
            last_played: Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap()
        }
    ]
}

fn steam_wishlist_fixture() -> Vec<WishlistedGame> {
    vec![
        WishlistedGame {
            id: GameId { app_id: 666 },
            wishlisted: Utc.with_ymd_and_hms(2012, 1, 1, 0, 0, 0).unwrap(),
            deleted: None,
        },
        WishlistedGame {
            id: GameId { app_id: 666666 },
            wishlisted: Utc.with_ymd_and_hms(2013, 1, 1, 0, 0, 0).unwrap(),
            deleted: None,
        },
    ]
}

fn notion_game_notes_fixture() -> Vec<GameNote> {
    vec![
        GameNote {
            id: "000001".to_string(),
            name: Some("Paint Drying Tycoon 2".to_string()),
            app_id: None,
            state: Some(GameState::NoRelease),
            tags: vec![],
            notes: Some("At least I'll get high off the fumes?".to_string()),
            rating: Some(1),
            created_time: Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap()
        }
    ]
}

#[tokio::test]
async fn test_basic_sync() {
    let conn_str = "host=localhost port=15432 user=tests password=test_admin dbname=rusteam_test";
    let mut db_client = db::connect(&conn_str).await;
    db::migrate(&mut db_client).await;

    let repo = Repo::new(db_client);

    let mut steam_client = MockSteamClient::new();

    steam_client
        .expect_get_all_games()
        .times(1)
        .returning(|| Ok(steam_app_list_fixture()));

    steam_client
        .expect_get_owned_games()
        .with(predicate::eq("STEAMID"))
        .times(1)
        .returning(|_| Ok(steam_owned_games_fixture()));

    steam_client
        .expect_get_wishlist()
        .with(predicate::eq("STEAMID"))
        .times(1)
        .returning(|_| Ok(steam_wishlist_fixture()));


    steam_client
        .expect_get_game_details()
        .with(predicate::function(
            |ids: &[GameId]| {
                let tracked_games: HashSet<GameId> = {
                    steam_owned_games_fixture()
                        .into_iter()
                        .chain(steam_wishlist_fixture().clone().into_iter().map(|item| item.id))
                        .collect()
                };
                ids.into_iter().cloned().collect::<HashSet<GameId>>() == tracked_games
            }
        ))
        .times(1)
        .returning(|_| Ok(steam_game_details_fixture()));

    steam_client
        .expect_get_played_games()
        .with(predicate::eq("STEAMID"))
        .times(1)
        .returning(|_| Ok(steam_played_games_fixture()));

    let mut notion_client = MockNotionClient::new();

    notion_client
        .expect_get_notes()
        .times(1)
        .returning(|| Ok(notion_game_notes_fixture()));

    // Should update the note with an app ID since it's discoverable from the above fixtures
    notion_client
        .expect_set_game_details()
        .with(
            predicate::eq("000001"),
            predicate::eq("654321"),
            predicate::eq("Paint Drying Tycoon 2")
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    let mut sync = Sync::new("STEAMID", repo, Box::new(steam_client), Box::new(notion_client));

    // Run the sync
    sync.sync_steam().await.unwrap();
    sync.sync_notion().await.unwrap();

    // TODO: Check the db has been updated as expected
}
