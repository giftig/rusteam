use std::collections::HashSet;
use std::path::PathBuf;

use tokio;
use tokio_postgres::Client;

use rusteam::cli::ignore::RunIgnoreGame;
use rusteam::config;
use rusteam::db;

fn config_file() -> String {
    "test/test-config.toml".to_string()
}

async fn get_ignored_games(db_client: &Client) -> HashSet<u32> {
    db_client
        .query("SELECT app_id FROM ignored_game", &[])
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<usize, i64>(0).try_into().unwrap())
        .collect()
}

#[tokio::test]
async fn test_ignore_game() {
    let cfg_file = config_file();

    let cmd = RunIgnoreGame {
        config_file: Some(PathBuf::from(&cfg_file)),
        games: vec!["123".to_string(), "456".to_string()],
    };

    cmd.run().await;

    let conf = config::read(Some(&cfg_file));
    let db_client = db::connect(&conf.db.connection_string()).await;

    let actual: HashSet<u32> = get_ignored_games(&db_client).await;
    let expected: HashSet<u32> = vec![123, 456].into_iter().collect();

    assert_eq!(actual, expected);
}
