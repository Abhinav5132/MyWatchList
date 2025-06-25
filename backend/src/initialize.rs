use std::{fs::File};
use std::io::Read;
use sqlx::Sqlite;
use serde::{Deserialize, Serialize,};
use sqlx::Pool;

#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeEntry {
    id: u64,
    title: Title,
    format: Option<String>,
    episodes: Option<u32>,
    status: Option<String>,
    season: Option<String>,
    seasonYear: Option<u32>,
    coverImage: CoverImage,
    duration: Option<u32>,  // in minutes
    popularity: Option<u64>,
    averageScore: Option<u32>,
    synonyms: Vec<String>,
    tags: Vec<Tag>,
    studios: Studios,
    relations: Relations,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Title {
    romaji: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CoverImage {
    extraLarge: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Studios {
    nodes: Vec<Studio>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Studio {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Relations {
    nodes: Vec<RelatedAnime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RelatedAnime {
    title: Title,
}

pub async fn initialize_database(connection: Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    println!("INITIALIZING");
    // opeing file for database json
    let mut file = File::open("anilist_data.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let parsed_json: Vec<AnimeEntry> = serde_json::from_str(&contents)?;
    let mut tx = connection.begin().await?;

    for anime in parsed_json.iter() {
    let title = anime.title.romaji.clone();
    let format = anime.format.clone().unwrap_or("Unknown".to_string());
    let episodes = anime.episodes.clone().unwrap_or(0);
    let status = anime.status.clone().unwrap_or("Unknown".to_string());
    let anime_season = anime.season.clone().unwrap_or("Unknown".to_string());
    let anime_year = anime.seasonYear.clone().unwrap_or(0);
    let duration = anime.duration.clone().unwrap_or(0); // always in seconds
    let score = anime.averageScore.clone().unwrap_or(0);
    let synonyms = anime.synonyms.clone();
    let studios: Vec<String> = anime.studios.nodes.iter().map(|studio| studio.name.clone()).collect();
    let related: Vec<String> = anime.relations.nodes.iter().map(|rel| rel.title.romaji.clone()).collect();
    let tags: Vec<String> = anime.tags.iter().map(|tag| tag.name.clone()).collect();
    let popularity = anime.popularity.clone().unwrap_or(0) as i64;
    let picture = anime.coverImage.extraLarge.clone();

    println!("{title}");
    let anime_id = sqlx::query(
        "INSERT INTO anime
        (title, format, episodes, status, anime_season, anime_year, picture, duration, score, popularity) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(title)
    .bind(format)
    .bind(episodes)
    .bind(status)
    .bind(anime_season)
    .bind(anime_year)
    .bind(picture)
    .bind(duration)
    .bind(score)
    .bind(popularity)
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
        .await?;}

    // inserting related anime
    for related_name in related.clone() {
    sqlx::query("INSERT INTO related_anime(anime_id, related_name) VALUES (?, ?)")
        .bind(anime_id)
        .bind(related_name)
        .execute(&mut *tx)
        .await?;
    }

    // inserting tags
    for tag in tags.clone() {
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
