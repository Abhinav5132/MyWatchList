use actix_cors::Cors;
use actix_web::{get, post, App, HttpServer, Responder, web};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, sqlite, *};

pub mod initialize;
pub use initialize::initialize_database;

pub mod details;
pub use details::get_details;

pub mod search;
pub use search::main_search;

pub mod login;
pub use login::login_fn;

pub mod sign_up;
pub use sign_up::sign_up_fn;

pub use search::trending_search;

pub mod authenticate;
use crate::backend::add_to_list::{check_if_an_anime_in_list, fetch_all_anime_from_list, fetch_all_lists, get_if_ranked};
pub use crate::backend::authenticate::*;

pub mod add_to_list;
pub use crate::backend::add_to_list::add_anime_to_list;
use crate::backend::sign_up::check_username_availability;
use crate::backend::user_profile::{change_pfp, get_user_details, logout};

pub mod user_profile;


// simple macro that takes a try get expression and a HttpResponse and 
// unwraps the result and returns the HttpResponse if the result is an error
#[macro_export]
macro_rules! try_or {
    ($expr:expr, $resp:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                dbg!(e);
                return $resp;
            }
        }
    };
}

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

#[derive(Serialize, Default, Deserialize, PartialEq)]
pub struct ReccomendResult{
    id: i32,
    title: String,
    picture: String,
    score: f32,
}


#[derive(Serialize, Default, Deserialize, PartialEq)]
pub struct RelatedAnime{
    id: i32,
    title: String,
    picture: String,
    RelationType: String
}

#[actix_web::main]
pub async fn setup_backend() -> std::io::Result<()> {
    let timestamp = chrono::Utc::now().timestamp();
    println!("{timestamp}");
    //database initializations
    let opt = sqlite::SqliteConnectOptions::new()
        .disable_statement_logging()
        .filename("anime.db") // for final relase make sure this is present in the same location as the binary or set a env variable with the file path.
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

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(connection.clone()))
            .service(main_search)
            .service(get_details)
            .service(trending_search)
            .service(login_fn)
            .service(sign_up_fn)
            .service(add_anime_to_list)
            .service(check_if_an_anime_in_list)
            .service(fetch_all_anime_from_list)
            .service(fetch_all_lists)
            .service(get_if_ranked)
            .service(check_username_availability)
            .service(issue_new_access_token)
            .service(get_user_details)
            .service(logout)
            .service(change_pfp)
    }).bind("127.0.0.1:3000")?
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


