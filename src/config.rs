use std::path::Path;

use home::home_dir;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_db")]
    pub db: Db,
    pub steam: Steam,
    pub notion: Notion
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub struct Db {
    #[serde_inline_default("localhost".to_string())]
    pub host: String,
    #[serde_inline_default("5432".to_string())]
    pub port: String,
    #[serde_inline_default("admin".to_string())]
    pub user: String,
    #[serde_inline_default("admin".to_string())]
    pub password: String,
    #[serde_inline_default("rusteam".to_string())]
    pub dbname: String
}

impl Db {
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host,
            self.port,
            self.user,
            self.password,
            self.dbname,
        )
    }
}

fn default_db() -> Db {
    Db {
        host: "localhost".to_string(),
        port: "5432".to_string(),
        user: "admin".to_string(),
        password: "admin".to_string(),
        dbname: "rusteam".to_string(),
    }
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub struct Steam {
    pub api_key: String,
    #[serde_inline_default("https://api.steampowered.com".to_string())]
    pub api_hoststring: String,
    #[serde_inline_default("https://store.steampowered.com".to_string())]
    pub store_hoststring: String,
    pub user_id: String,
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub struct Notion {
    pub api_key: String,
    pub database_id: String,
    #[serde_inline_default("https://api.notion.com".to_string())]
    pub api_hoststring: String,
}


pub fn read<P: AsRef<Path>>(path: Option<&P>) -> Config {
    let f = match path {
        Some(p) => p.as_ref().to_owned(),
        None => {
            let mut p = home_dir().expect("Failed to load config: could not determine home dir");
            p.push(".rusteam/config.toml");
            p
        }
    };

    let raw = std::fs::read_to_string(&f).expect("Failed to read config file");
    toml::from_str(&raw).expect("Failed to parse config file")
}
