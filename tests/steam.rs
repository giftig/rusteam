mod utils;

use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use tokio;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

use rusteam::models::game::{GameId, SteamPlaytime, WishlistedGame};
use rusteam::models::steam::SteamAppIdPair;
use rusteam::steam::{
    SteamClient,
    SteamAppsServiceHandling,
    SteamPlayerServiceHandling,
    SteamWishlistHandling
};

#[tokio::test]
async fn test_get_owned_games() {
    let mock_steam = MockServer::start().await;
    let response = utils::fixture("owned-games/owned-games-1.json");

    Mock::given(method("GET"))
        .and(path("/IPlayerService/GetOwnedGames/v0001/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(response.as_bytes(), "application/json")
        )
        .mount(&mock_steam)
        .await;

    let steam_client = SteamClient::new(
        "STEAM API KEY",
        &format!("http://{}", &mock_steam.address()),
        &format!("http://{}", &mock_steam.address())
    );

    let expected = vec![
        GameId { app_id: 666 },
        GameId { app_id: 1337 },
        GameId { app_id: 9876 },
    ];
    let actual = steam_client.get_owned_games("TEST API KEY").unwrap();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_get_played_games() {
    let mock_steam = MockServer::start().await;
    let response = utils::fixture("owned-games/owned-games-1.json");

    Mock::given(method("GET"))
        .and(path("/IPlayerService/GetOwnedGames/v0001/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(response.as_bytes(), "application/json")
        )
        .mount(&mock_steam)
        .await;

    let steam_client = SteamClient::new(
        "STEAM API KEY",
        &format!("http://{}", &mock_steam.address()),
        &format!("http://{}", &mock_steam.address())
    );

    let expected = vec![
        SteamPlaytime {
            id: GameId { app_id: 666 },
            playtime: Duration::new(100 * 60, 0),
            last_played: DateTime::UNIX_EPOCH
        },
        SteamPlaytime {
            id: GameId { app_id: 1337 },
            playtime: Duration::new(1000 * 60, 0),
            last_played: DateTime::UNIX_EPOCH
        },
        SteamPlaytime {
            id: GameId { app_id: 9876 },
            playtime: Duration::new(10000 * 60, 0),
            last_played: Utc.with_ymd_and_hms(2024, 3, 1, 6, 5, 4).unwrap()
        }
    ];
    let actual = steam_client.get_played_games("TEST API KEY").unwrap();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_get_all_games() {
    let mock_steam = MockServer::start().await;
    let response = utils::fixture("steam-games/steam-games-1.json");

    Mock::given(method("GET"))
        .and(path("/ISteamApps/GetAppList/v2/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(response.as_bytes(), "application/json")
        )
        .mount(&mock_steam)
        .await;

    let steam_client = SteamClient::new(
        "STEAM API KEY",
        &format!("http://{}", &mock_steam.address()),
        &format!("http://{}", &mock_steam.address())
    );

    // TODO: DRY, reuse the identical fixture list from unit tests
    let expected = vec![
        SteamAppIdPair { appid: 666, name: "Game Buying Simulator 2024".to_string() },
        SteamAppIdPair { appid: 1337, name: "Final Fantasy MMLXVII".to_string() },
        SteamAppIdPair { appid: 654321, name: "Paint Drying Tycoon 2".to_string() },
    ];
    let actual = steam_client.get_all_games().unwrap();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_get_wishlist() {
    let mock_steam = MockServer::start().await;
    let response = utils::fixture("wishlist/wishlist-1.json");

    Mock::given(method("GET"))
        .and(path("/IWishlistService/GetWishlist/v1/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(response.as_bytes(), "application/json")
        )
        .mount(&mock_steam)
        .await;

    let steam_client = SteamClient::new(
        "STEAM API KEY",
        &format!("http://{}", &mock_steam.address()),
        &format!("http://{}", &mock_steam.address())
    );

    let expected = vec![
        WishlistedGame {
            id: GameId { app_id: 1093810 },
            wishlisted: Utc.with_ymd_and_hms(2019, 10, 11, 6, 24, 8).unwrap(),
            deleted: None,
        },
        WishlistedGame {
            id: GameId { app_id: 1125510 },
            wishlisted: Utc.with_ymd_and_hms(2024, 2, 9, 8, 26, 41).unwrap(),
            deleted: None,
        },
        WishlistedGame {
            id: GameId { app_id: 1145350 },
            wishlisted: Utc.with_ymd_and_hms(2023, 12, 6, 7, 48, 49).unwrap(),
            deleted: None,
        },
    ];
    let mut actual = steam_client.get_wishlist("TEST API KEY").unwrap();
    actual.sort_by_key(|item| item.id.app_id);

    assert_eq!(actual, expected);
}
