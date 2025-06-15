use std::{fs::File};
use std::io::Read;
use actix_web::rt::System;
use sqlx::Sqlite;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite, Executor, Pool};

#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeDatabase{
    data: Vec<AnimeEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeEntry {
    sources: Vec<String>,
    title: String,
    #[serde(rename = "type")]
    anime_type: String,
    episodes: Option<u32>,
    status: String,
    animeSeason: Option<AnimeSeason>,
    picture: String,
    thumbnail: String,
    duration: Option<AnimeDuration>,
    score: Option<AnimeScore>,
    synonyms: Vec<String>,
    studios: Vec<String>,
    producers: Vec<String>,
    #[serde(default)]
    related: Vec<String>,
    tags: Vec<String>

}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeSeason{
    season: String,
    year: Option<u32>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeDuration{
    value: u32,
    unit: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeScore{
    arithmeticGeometricMean: Option<f64>,
    arithmeticMean: Option<f64>,
    median: Option<f64>,
}

pub async fn initialize_database(connection: Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    println!("INITIALIZING");
    let mut file = File::open("anime-offline-database.json")?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let parsed_json: AnimeDatabase = serde_json::from_str(&contents)?;

    let mut tx = connection.begin().await?;

    for anime in parsed_json.data.iter() {
    let title = anime.title.clone();
    let format = anime.anime_type.clone();
    let episodes = anime.episodes.clone();
    let status = anime.status.clone();

    let anime_season = anime.animeSeason.as_ref()
        .map(|s| s.season.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let anime_year = anime.animeSeason.as_ref()
        .and_then(|s| s.year)
        .unwrap_or(0);
    
    let picture = anime.picture.clone();
    let thumbnail = anime.thumbnail.clone();

    let duration = anime.duration.as_ref().map(|d| d.value).unwrap_or(0); // always in seconds
    
    let score = anime.score.as_ref().and_then(|s| s.arithmeticGeometricMean).unwrap_or(0.0);
    
    let synonyms = anime.synonyms.clone();
    let studios = anime.studios.clone();
    let producers = anime.producers.clone();
    let related = anime.related.clone();
    let tags = anime.tags.clone();
    println!("{title}");
    let anime_id = sqlx::query(
        "INSERT INTO anime
        (title, format, episodes, status, anime_season, anime_year, picture, thumbnail, duration, score) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(title)
    .bind(format)
    .bind(episodes)
    .bind(status)
    .bind(anime_season)
    .bind(anime_year)
    .bind(picture)
    .bind(thumbnail)
    .bind(duration)
    .bind(score)
    .execute(&mut *tx).await?
    .last_insert_rowid();

    // inserting synonyms
    for synonym in synonyms {
        sqlx::query("INSERT INTO synonyms(anime_id, synonym) VALUES (?, ?)")
            .bind(anime_id)
            .bind(synonym)
            .execute(&mut *tx).await?;
    }

    // inserting studios
    for studio in studios {
    // First insert the studio
    sqlx::query("INSERT OR IGNORE INTO studios (name) VALUES (?)")
        .bind(&studio)
        .execute(&mut *tx)
        .await?;

    // Then get its ID
    let studio_id: i64 = sqlx::query_scalar("SELECT id FROM studios WHERE name = ?")
        .bind(&studio)
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO anime_studio(anime_id, studio_id) VALUES (?, ?)")
        .bind(anime_id)
        .bind(studio_id)
        .execute(&mut *tx)
        .await?;
}

    // inserting producers
    for producer in producers {
    sqlx::query("INSERT OR IGNORE INTO producers (name) VALUES (?)")
        .bind(&producer)
        .execute(&mut *tx)
        .await?;

    let producer_id: i64 = sqlx::query_scalar("SELECT id FROM producers WHERE name = ?")
        .bind(&producer)
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO anime_producers(anime_id, producer_id) VALUES (?, ?)")
        .bind(anime_id)
        .bind(producer_id)
        .execute(&mut *tx)
        .await?;
}


    // inserting related anime (assuming related contains anime IDs)
    for related_anime_id in related {
        sqlx::query("INSERT INTO related_anime(anime_id, related_anime) VALUES (?, ?)")
            .bind(anime_id)
            .bind(related_anime_id)
            .execute(&mut *tx).await?;
    }

    // inserting tags
    for tag in tags {
    sqlx::query("INSERT OR IGNORE INTO tags (tag) VALUES (?)")
        .bind(&tag)
        .execute(&mut *tx)
        .await?;

    let tag_id: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE tag = ?")
        .bind(&tag)
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO anime_tags(anime_id, tag_id) VALUES (?, ?)")
        .bind(anime_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
}
}
    tx.commit().await?;
    Ok(())

}
