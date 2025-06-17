use actix_cors::Cors;
use actix_web::{App, HttpServer, Responder, get, web};
use serde::Deserialize;
use sqlx::{Pool, Sqlite, sqlite, *};

mod initialize;
use initialize::initialize_database;

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}
fn main() {
    setup_backend();
}

#[actix_web::main]
async fn setup_backend() -> std::io::Result<()> {
    let opt = sqlite::SqliteConnectOptions::new()
        .filename("anime.db")
        .create_if_missing(true);

    let connection = sqlite::SqlitePool::connect_with(opt).await.unwrap();

    let schema = std::fs::read_to_string("anime.sql").unwrap();
    connection.execute(&*schema).await.unwrap();

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
            .service(search)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

#[get("/search")]
async fn search(db: web::Data<Pool<Sqlite>>, query: web::Query<SearchQuery>) -> impl Responder {
    let title = format!("%{}%", query.query);
    match sqlx::query("SELECT title FROM anime WHERE title LIKE ? LIMIT 5")
        .bind(&title)
        .fetch_all(db.get_ref())
        .await
    {
        Ok(rows) => {
            let names: Vec<String> = rows.into_iter()
            .filter_map(|r| r.try_get::<String,_>("title")
            .ok())
            .collect();
            if names.is_empty() {
                // this later needs to fetch everything in the anime table using this anime id so we can create anime structs
                let mut names:Vec<String> = vec![];
                match sqlx::query(
                    "
                SELECT title FROM anime 
                JOIN synonyms ON anime.id = synonyms.anime_id
                WHERE synonyms.synonym LIKE ? LIMIT 5
                ",
                )
                .bind(&title)
                .fetch_all(db.as_ref())
                .await
                {
                    Ok(rows) => {
                        let names: Vec<String> = rows.into_iter()
                        .filter_map(|r| r.try_get::<String, _>("title").ok()).collect();
                        return web::Json(names);
                    }
                    Err(_) =>  {return web::Json(vec!["error querying the database".to_string()]);}
                }
            }
            web::Json(names)
        }
        Err(_) => web::Json(vec!["error querying the database".to_string()]),
    }
}
