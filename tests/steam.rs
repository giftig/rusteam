//use tokio;
//use wiremock::{MockServer, Mock, ResponseTemplate};

//#[tokio::test]
//async fn test_steam_client() {
//    let mock_steam_api = MockServer::start().await;
//    let mock_steam_store = MockServer::start().await;
//    let steam_client = SteamClient::new(
//        "STEAM API KEY",
//        &format!("http://{}", &mock_steam_api.address()),
//        &format!("http://{}", &mock_steam_store.address())
//    );

//    assert_eq!(1, 2);
//}
