use std::env;

use actix_web::web::Data;
use actix_web::{get, post, App, HttpServer, Responder, web};
use actix_cors::Cors;
use dotenvy::dotenv;
use reqwest::header;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, postgres,*};
use env_logger::Env;
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
pub mod friends;

pub mod authenticate;
use crate::add_to_list::{check_if_an_anime_in_list, edit_watch_list, fetch_all_anime_from_list, fetch_all_lists, get_if_ranked, get_list_details, remove_watch_list};
pub use crate::authenticate::*;

pub mod add_to_list;
pub use crate::add_to_list::add_anime_to_list;
use crate::details::{ReccomendResult, RelatedAnime};
use crate::friends::get_all_friends;
use crate::sign_up::check_username_availability;
use crate::user_profile::{change_email, change_password, change_pfp, change_username, get_user_details, logout};

pub mod user_profile;
pub mod AnimeStructs;
pub mod update_database;

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
    largeImage: Option<String>,
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
    largeImage: String,
    duration: i32,
    score: f32,
    studio: Option<Vec<String>>,
    synonyms: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    recommendations: Vec<ReccomendResult>,
    related_anime: Vec<RelatedAnime>
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let timestamp = chrono::Utc::now().timestamp();
    println!("{timestamp}");
    //database initializations
    dotenv().ok();
    let database_url = match env::var("DATABASE_URL"){
        Ok(url) => url,
        Err(e) => {
            dbg!(e);
            panic!("DATABASE URL NOT SET")
        }
    };

    let connection = match PgPool::connect(&database_url).await{
        Ok(a)=> a,
        Err(e)=>{
            dbg!(e);
            panic!("UNABLE TO CONNECT TO THE DATABASE. PLEASE MAKE SURE IT IS RUNNING.")
        }
    };

    let schema = std::fs::read_to_string("anime.sql").unwrap_or_default();
    match connection.execute(&*schema).await {
        Ok(c) => c,
        Err(r) => {
            dbg!(r); 
            panic!("Failed to execure schema.");
        }
    };
    
    let _ = sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&connection)
        .await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anime")
        .fetch_one(&connection)
        .await
        .unwrap_or(0);

    let db:Data<Pool<Postgres>> = web::Data::new(connection.clone());
    if count == 0 {
        match initialize_database(db.clone()).await {
            Ok(_) => println!("Database initialized successfully"),
            Err(e) => eprintln!("Failed to initialize database: {}", e),
        };
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default())
            .app_data(db.clone())
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
            .service(change_username)
            .service(change_email)
            .service(verify_entered_password)
            .service(change_password)
            .service(get_list_details)
            .service(edit_watch_list)
            .service(remove_watch_list)
            .service(get_all_friends)
    }).bind("127.0.0.1:3000")?
    .run()
    .await
}


