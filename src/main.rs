use sqlx::{sqlite, Executor};

mod Initialize;
use Initialize::initialize_database;


#[tokio::main]
async fn main() {
    let opt = sqlite::SqliteConnectOptions::new()
        .filename("anime.db")
        .create_if_missing(true);

    let connection = sqlite::SqlitePool::connect_with(opt).await.unwrap();

    let schema = std::fs::read_to_string("anime.sql").unwrap();
    connection.execute(&*schema).await.unwrap();

    let count:i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anime").fetch_one(&connection)
    .await.unwrap_or(0);
    if count == 0 {
    match initialize_database(connection).await{
        Ok(_) => println!("Database initialized successfully"),
        Err(e) => eprintln!("Failed to initialize database: {}", e),
    };
    }
}

