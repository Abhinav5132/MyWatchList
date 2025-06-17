use sqlx::{sqlite, Executor, Pool};

mod Initialize;
use Initialize::initialize_database;
mod basic_search;
use basic_search::search_main;
mod global_db;
use global_db::set_db_pool;

use sqlx::Sqlite;
use std::sync::Arc;


fn main() {

    let connection= setup_database();
    set_db_pool(connection);
    search_main();

}

#[tokio::main]
async fn setup_database() -> Arc<Pool<Sqlite>> {
    let opt = sqlite::SqliteConnectOptions::new()
    .filename("anime.db")
    .create_if_missing(true);

    let connection = sqlite::SqlitePool::connect_with(opt).await.unwrap();

    let schema = std::fs::read_to_string("anime.sql").unwrap();
    connection.execute(&*schema).await.unwrap();

    let count:i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anime").fetch_one(&connection)
    .await.unwrap_or(0);
    if count == 0 {
    match initialize_database(connection.clone()).await{
        Ok(_) => println!("Database initialized successfully"),
        Err(e) => eprintln!("Failed to initialize database: {}", e),
    };
    }
    Arc::new(connection)
}