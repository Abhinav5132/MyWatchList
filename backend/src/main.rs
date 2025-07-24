use actix_cors::Cors;
use actix_web::{App, HttpServer, Responder, get, web};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, sqlite, *};


pub mod initialize;
use initialize::initialize_database;

pub mod details;
use details::get_details;

pub mod search;
use search::main_search;

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[derive(Serialize)]
struct AnimeResult {
    id: i32,
    title: String,
    picture: Option<String>,
}
#[derive(Serialize, Default, Deserialize)]
struct FullAnimeResult {
    title: String,
    format: String,
    description: String,
    episodes: i32,
    status:String,
    anime_season: String,
    anime_year: i32,
    picture: String,
    duration: i32,
    score: f32,
    trailer_url: String,
    studio: Option<Vec<String>>,
    synonyms: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    recommendations: Vec<ReccomendResult>,
    related_anime: Vec<RelatedAnime>
}

#[derive(Serialize, Default, Deserialize)]
struct ReccomendResult{
    id: i32,
    title: String,
    picture: String,
    score: f32,
}


#[derive(Serialize, Default, Deserialize)]
struct RelatedAnime{
    id: i32,
    title: String,
    picture: String,
    RelationType: String
}

fn main() {
    dotenvy::dotenv().ok();

    let result = setup_backend();
    if !result.is_err(){
        println!("unable to start backend");
    }
}

#[actix_web::main]
async fn setup_backend() -> std::io::Result<()> {
    let opt = sqlite::SqliteConnectOptions::new()
        .filename("anime.db")
        .create_if_missing(true);

    let connection = sqlite::SqlitePool::connect_with(opt).await.unwrap();
    let schema = std::fs::read_to_string("anime.sql").unwrap();
    connection.execute(&*schema).await.unwrap();
    
    let _ = sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&connection)
        .await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anime")
        .fetch_one(&connection)
        .await
        .unwrap_or(0);
    if count == 0 {
        match initialize_database(connection.clone()).await {
            Ok(_) => println!("Database initialized successfully"),
            Err(e) => eprintln!("Failed to initialize database: {}", e),
        };
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(connection.clone()))
            .service(main_search)
            .service(get_details)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}


