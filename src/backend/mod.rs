#![allow(clippy::redundant_field_names)]
pub use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix_cors::Cors;
use actix_web::web::{Data, Json};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use env_logger::Env;
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
use tokio::time::{Sleep, sleep};
pub mod friends;

pub mod authenticate;
pub mod friends_test;
use crate::backend::add_to_list::{
    check_if_an_anime_in_list, edit_watch_list, fetch_all_anime_from_list, fetch_all_lists,
    get_if_ranked, get_list_details, remove_watch_list,
};
pub use crate::backend::authenticate::*;

pub mod add_to_list;
pub use crate::backend::add_to_list::add_anime_to_list;
use crate::backend::details::{ReccomendResult, RelatedAnime};
use crate::backend::friends::get_all_friends;
use crate::backend::sign_up::{
    AuthResponse, check_availability, check_username_availability,
};
use crate::backend::user_profile::{
    change_email, change_password, change_pfp, change_username, get_user_details, logout,
};
use crate::backend::verification_service::{TokenVerifier, VerificationService};
pub mod partial_update;
pub mod AnimeStructs;
pub mod update_database;
pub mod user_profile;
pub mod verification_service;
pub mod full_update;
use crate::backend::update_database::*;

#[post("/issue_new_access")]
pub async fn issue_new_access_token(
    db: Data<Pool<Sqlite>>,
    refresh_token: Json<AuthResponse>,
) -> HttpResponse {
    dotenvy::dotenv().ok();
    match sqlx::query(
        "SELECT id 
    FROM user 
    WHERE user_refresh_token = ?;
    ",
    )
    .bind(&refresh_token.refresh_token)
    .fetch_one(db.as_ref())
    .await
    {
        Ok(row) => {
            let user_id: i64 = match row.try_get("id") {
                Ok(u) => u,
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::Unauthorized().into();
                }
            };
            let access_token = match generate_access_token(user_id).await {
                Ok(token) => {
                    let _ = match sqlx::query(
                        "
                    UPDATE user SET user_access_token = ? WHERE id = ?;
                    ",
                    )
                    .bind(&token)
                    .bind(user_id)
                    .execute(db.as_ref())
                    .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            dbg!(e);
                            return HttpResponse::Unauthorized().into();
                        }
                    };
                    token
                }
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::Unauthorized().into();
                }
            };
            HttpResponse::Ok().json(IssueNewAccess {
                access_token: access_token,
                expiry: (chrono::Utc::now() + chrono::Duration::minutes(3)).timestamp() as u64,
            })
        }
        Err(e) => {
            dbg!(e);
            HttpResponse::Unauthorized().into()
        }
    }
}

//tests
pub mod add_to_list_test;

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
    status: String,
    anime_season: String,
    anime_year: i32,
    largeImage: String,
    duration: i32,
    score: f32,
    studio: Option<Vec<String>>,
    synonyms: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    recommendations: Vec<ReccomendResult>,
    related_anime: Vec<RelatedAnime>,
}

pub async fn start_backend_updater(db: Data<Pool<Sqlite>>){
    let interval = Duration::from_secs(120);
    tokio::spawn(async move {
        loop {
            if let Err(e) = update_database(db.clone(), interval).await {
                dbg!(e);
            }
        }
    });
}

pub async fn setup_db() -> Data<Pool<Sqlite>> {
    //database initializations
    let opt = sqlite::SqliteConnectOptions::new()
        .disable_statement_logging()
        .filename("anime.db")
        .create_if_missing(true);

    let connection = match sqlite::SqlitePool::connect_with(opt).await {
        Ok(c) => c,
        Err(e) => {
            dbg!(e);
            panic!("Failed to establish connection to the db")
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

    let db = web::Data::new(connection.clone());

    db
}

#[actix_web::main]
pub async fn setup_backend(tx: Sender<()>) -> std::io::Result<()> {
    let timestamp = chrono::Utc::now().timestamp();
    println!("{timestamp}");
    let db = setup_db().await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anime")
        .fetch_one(db.as_ref())
        .await
        .unwrap_or(0);

    if count == 0 {
        match initialize_database(db.clone()).await {
            Ok(_) => println!("Database initialized successfully"),
            Err(e) => eprintln!("Failed to initialize database: {}", e),
        };
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();
    let verifier: Arc<dyn TokenVerifier> = Arc::new(VerificationService { db: db.clone() });

    start_backend_updater(db.clone()).await;

    let _ = tx.send(());

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(db.clone())
            .app_data(Data::from(verifier.clone()))
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
            .service(check_availability)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
