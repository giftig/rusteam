use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDateTime, Utc};
use thiserror::Error;
use tokio_postgres::{Client, Error as PgError};

use crate::models::game::OwnedGame;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Error connecting to the database: {0}")]
    Postgres(#[from] PgError),
}

pub type Result<T> = std::result::Result<T, RepoError>;

pub struct Repo {
    db: Client
}

impl Repo {
    pub fn new(db: Client) -> Repo {
        Repo { db: db }
    }

    async fn get_unknown_app_ids(&self, app_ids: &HashSet<i32>) -> Result<HashSet<i32>> {
        let q = "SELECT app_id FROM steam_game WHERE app_id = ANY ($1)";
        let known = self.db
            .query(q, &[&Vec::from_iter(app_ids)])
            .await?
            .into_iter()
            .map(|row| row.get(0))
            .collect::<HashSet<i32>>();

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
    async fn insert_owned_games(&self, games: &Vec<OwnedGame>) -> Result<()>;
}

impl SteamGamesHandling for Repo {
    async fn insert_steam_games<T: AsRef<str>>(&self, games: HashMap<u32, T>) -> Result<()> {
        // FIXME: May need to change the type in postgres; u32 is bigger than i32
        let ids: HashSet<i32> = games.keys().cloned().map(|i| i32::try_from(i).unwrap()).collect();
        let unknown_ids = self.get_unknown_app_ids(&ids).await?;

        // TODO: Use a transaction here:
        // https://docs.rs/tokio-postgres/latest/tokio_postgres/struct.Transaction.html
        let q = "INSERT INTO steam_game (app_id, name) VALUES ($1, $2)";
        println!("Inserting {} new steam games into steam_game table", unknown_ids.len());

        for id in unknown_ids {
            let name = games.get(&u32::try_from(id).unwrap());

            if let Some(n) = name {
                self.db.execute(q, &[&id, &n.as_ref()]).await?;
            }
        }
        Ok(())
    }
}

impl OwnedGamesHandling for Repo {
    async fn insert_owned_games(&self, games: &Vec<OwnedGame>) -> Result<()> {
        // TODO: Unless I get an accurate purchase date, I can't use this approach
        // I'll have to fall back on checking IDs instead, annoyingly
        let last_updated = self.get_owned_games_last_updated().await?;

        let q = "INSERT INTO owned_game(app_id, purchased) VALUES($1, $2) ON CONFLICT DO NOTHING";

        let new_games = last_updated
            .map(|u| games.iter().cloned().filter(|g| g.purchased.naive_utc() > u).collect())
            .unwrap_or(games.to_owned());

        println!("Inserting {} new owned games into owned_game table", new_games.len());
        for g in new_games.into_iter() {
            if let Err(e) = self.db.execute(q, &[&i32::try_from(g.app_id).unwrap(), &g.purchased.naive_utc()]).await {
                eprintln!("Couldn't insert game {}: {}", g.app_id, e);
            };
        }
        Ok(())
    }
}
