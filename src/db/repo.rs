use std::collections::{HashMap, HashSet};
use std::num::TryFromIntError;

use chrono::{NaiveDateTime, Utc};
use thiserror::Error;
use tokio_postgres::{Client, Error as PgError};

use crate::models::game::GameId;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Error connecting to the database: {0}")]
    Postgres(#[from] PgError),
    #[error("Error converting integer types: {0}")]
    IntConversion(#[from] TryFromIntError),
}

pub type Result<T> = std::result::Result<T, RepoError>;

pub struct Repo {
    db: Client
}

impl Repo {
    pub fn new(db: Client) -> Repo {
        Repo { db: db }
    }

    async fn get_unknown_app_ids(&self, app_ids: &HashSet<u32>) -> Result<HashSet<u32>> {
        let q = "SELECT app_id FROM steam_game WHERE app_id = ANY ($1)";
        let known: HashSet<u32> = self.db
            .query(q, &[&Vec::from_iter(app_ids.iter().map(|&id| i64::from(id)))])
            .await?
            .into_iter()
            .map(|row| row.get::<usize, i64>(0).try_into().unwrap())
            .collect();

        Ok(app_ids - &known)
    }

    async fn get_owned_games_last_updated(&self) -> Result<Option<NaiveDateTime>> {
        let q = "SELECT purchased FROM owned_game ORDER BY purchased DESC LIMIT 1";
        let res = self.db.query(q, &[]).await?;

        Ok(res.first().map(|row| row.get(0)))
    }
}

pub trait SteamGamesHandling {
    async fn insert_steam_games<T: AsRef<str>>(&self, games: HashMap<u32, T>) -> Result<()>;
}

pub trait OwnedGamesHandling {
    async fn insert_owned_games(&self, games: &[GameId]) -> Result<()>;
}

impl SteamGamesHandling for Repo {
    async fn insert_steam_games<T: AsRef<str>>(&self, games: HashMap<u32, T>) -> Result<()> {
        let ids: HashSet<u32> = games.keys().cloned().collect();
        let unknown_ids: HashSet<u32> = self.get_unknown_app_ids(&ids).await?;

        // TODO: Use a transaction here:
        // https://docs.rs/tokio-postgres/latest/tokio_postgres/struct.Transaction.html
        let q = "INSERT INTO steam_game (app_id, name) VALUES ($1, $2)";
        println!("Inserting {} new steam games into steam_game table", unknown_ids.len());

        for id in unknown_ids {
            let name = games.get(&u32::try_from(id).unwrap());

            if let Some(n) = name {
                self.db.execute(q, &[&i64::from(id), &n.as_ref()]).await?;
            }
        }
        Ok(())
    }
}

impl OwnedGamesHandling for Repo {
    async fn insert_owned_games(&self, games: &[GameId]) -> Result<()> {
        let now = Utc::now().naive_utc();
        let q = "INSERT INTO owned_game(app_id, first_recorded) VALUES($1, $2) ON CONFLICT DO NOTHING";

        println!("Inserting owned games into owned_game table");

        let mut row_count: u64 = 0;

        for id in games.into_iter() {
            match self.db.execute(q, &[&Into::<i64>::into(id.clone()), &now]).await {
                Ok(c) => row_count += c,
                Err(e) => eprintln!("Couldn't insert game {}: {}", id.app_id, e),
            }
        }

        println!("Inserted {} new owned games into owned_game table", row_count);
        Ok(())
    }
}
