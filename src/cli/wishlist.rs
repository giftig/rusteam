use clap::Parser;

use crate::config;
use crate::db;

#[derive(Debug, Parser)]
pub(super) struct ImportFromFile {
    cookie: String,
}

impl ImportFromFile {
    pub(super) async fn run(&self) {
        let conf = config::read();
        let mut db_client = db::connect(&conf.db.connection_string()).await;
        db::migrate(&mut db_client).await;

        panic!("Not yet implemented!");
    }
}
