pub mod migrations;
pub mod repo;
pub mod sync;

use tokio_postgres::{Client, NoTls};

/// Run migrations or panic. Most subcommands should run this immediately on execution.
pub(crate) async fn migrate(db_client: &mut Client) -> () {
    match migrations::run(db_client).await {
        Ok(report) => {
            let count = report.applied_migrations().len();
            println!("Successfully ran {} migrations.", count);
        }
        Err(e) => {
            panic!("Migration error: {}", e);
        }
    }
}

/// Connect to postgres and report any connection errors as async
pub(crate) async fn connect(connection_string: &str) -> Client {
    let (db_client, conn) = tokio_postgres::connect(connection_string, NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            panic!("connection error: {}", e);
        }
    });

    db_client
}
