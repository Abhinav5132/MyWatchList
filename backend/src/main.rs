use actix_cors::Cors;
use actix_web::{get, post, App, HttpServer, Responder, web};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, sqlite, *};


pub mod initialize;
use initialize::initialize_database;

pub mod details;
use details::get_details;

pub mod search;
use search::main_search;

use crate::search::trending_search;

pub mod authenticate;
use crate::authenticate::*;

pub mod login;
pub mod sign_up;

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
    title_romanji: String,
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
    let timestamp = chrono::Utc::now().timestamp();
    println!("{timestamp}");
    //database initializations
    let opt = sqlite::SqliteConnectOptions::new()
        .filename("anime.db")
        .create_if_missing(true);

    let connection = sqlite::SqlitePool::connect_with(opt).await.unwrap();
    let schema = std::fs::read_to_string("anime.sql").unwrap_or_default();
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
    
    //https server initializations
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("keys/key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("keys/cert.pem").unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(connection.clone()))
            .service(main_search)
            .service(get_details)
            .service(trending_search)
    }).bind_openssl("127.0.0.1:3000", builder)?
    .run()
    .await
    /* 
    use for production
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default())
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION])
            .expose_headers(vec![header::AUTHORIZATION])
            .app_data(web::Data::new(connection.clone()))
            .service(main_search)
            .service(get_details)
            .service(trending_search)
    }).bind_openssl("127.0.0.1:3000", builder)?
    .run()
    .await*/
}


