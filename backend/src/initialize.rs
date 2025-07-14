#![allow(non_snake_case)]
use std::{fs::File};
use std::io::Read;
use sqlx::Sqlite;
use serde::{Deserialize, Serialize};
use sqlx::Pool;



#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeEntry {
    id: u64,
    title: String,
    description: Option<String>,
    format: Option<String>,
    episodes: Option<u32>,
    status: Option<String>,
    season: Option<String>,
    seasonYear: Option<u32>,
    coverImage: Option<String>,
    duration: Option<u32>,  // in minutes
    popularity: Option<u64>,
    averageScore: Option<u32>,
    synonyms: Vec<String>,
    tags: Vec<String>,
    studios: Vec<String>,
    relations: Vec<RelatedAnime>,
    trailer: Option<String>,
    characters: Vec<Characters>,
    recommendations: Vec<Recommendation>,
    banner_image: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Relations {
    nodes: Vec<RelatedAnime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RelatedAnime {
    title: String,
    r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)] 
pub struct Characters{
    name: String,
    role: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Recommendation{
    title: String
}

pub async fn initialize_database(connection: Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    println!("INITIALIZING");
    // opeing file for database json
    let mut file = File::open("anilist_data.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let parsed_json: Vec<AnimeEntry> = serde_json::from_str(&contents)?;
    let mut tx = connection.begin().await?;

    use std::collections::HashMap;
    let mut studio_cache: HashMap<String, i64> = HashMap::new();
    let mut tag_cache: HashMap<String, i64> = HashMap::new();
    let mut character_cache: HashMap<String, i64> = HashMap::new();

    for anime in parsed_json.iter() {

    let title = &anime.title;
    let description = anime.description.as_deref().unwrap_or("");
    let format = anime.format.as_deref().unwrap_or("Unknown");
    let episodes = anime.episodes.unwrap_or(0);
    let status = anime.status.as_deref().unwrap_or("Unknown");
    let anime_season = anime.season.as_deref().unwrap_or("Unknown");
    let anime_year = anime.seasonYear.unwrap_or(0);
    let duration = anime.duration.unwrap_or(0); 
    let score = anime.averageScore.unwrap_or(0);
    let popularity = anime.popularity.unwrap_or(0) as i64;
    let picture = anime.coverImage.as_deref().unwrap_or("none");
    let trailerUrl = anime.trailer.as_deref().unwrap_or("none");
    let banner_image = anime.banner_image.as_deref().unwrap_or("none");
    println!("{title}");
    let anime_id = sqlx::query(
        "INSERT INTO anime
        (title, description, format, episodes, status, anime_season, anime_year, picture, duration, averageScore, popularity, banner_image, trailer_url) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(title)
    .bind(description)
    .bind(format)
    .bind(episodes)
    .bind(status)
    .bind(anime_season)
    .bind(anime_year)
    .bind(picture)
    .bind(duration)
    .bind(score)
    .bind(popularity)
    .bind(banner_image)
    .bind(trailerUrl)
    .execute(&mut *tx).await?
    .last_insert_rowid();

    // inserting synonyms
    for synonym in &anime.synonyms {
        sqlx::query("INSERT INTO synonyms(anime_id, synonym) VALUES (?, ?)")
            .bind(anime_id)
            .bind(synonym)
            .execute(&mut *tx).await?;
    }

    // inserting studios
    for studio in &anime.studios{
        let studio_id = if let Some(id) = studio_cache.get(studio) {
                    *id
                } else {
                    sqlx::query("INSERT OR IGNORE INTO studios (name) VALUES (?)")
                        .bind(studio)
                        .execute(&mut *tx)
                        .await?;
                    let id: i64 = sqlx::query_scalar("SELECT id FROM studios WHERE name = ?")
                        .bind(studio)
                        .fetch_one(&mut *tx)
                        .await?;
                    studio_cache.insert(studio.clone(), id);
                    id
                };
        sqlx::query("INSERT INTO anime_studio(anime_id, studio_id) VALUES (?, ?)")
            .bind(anime_id)
            .bind(studio_id)
            .execute(&mut *tx)
            .await?;
        }

    // inserting related anime
    //let related: Vec<String> = anime.relations.nodes.iter().map(|rel| rel.title.romaji.clone()).collect();
    for related_anime in &anime.relations {
    sqlx::query("INSERT INTO related_anime(anime_id, related_name, relation_type) VALUES (?, ?, ?)")
        .bind(anime_id)
        .bind(related_anime.title.clone())
        .bind(related_anime.r#type.clone())
        .execute(&mut *tx)
        .await?;
    }

    // inserting tags
    for tag in &anime.tags {
        let tag_id = if let Some(id) = tag_cache.get(tag) {
                    *id
            } else {
                sqlx::query("INSERT OR IGNORE INTO tags (tag) VALUES (?)")
                    .bind(tag)
                    .execute(&mut *tx)
                    .await?;
                let id: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE tag = ?")
                    .bind(tag)
                    .fetch_one(&mut *tx)
                    .await?;
                tag_cache.insert(tag.clone(), id);
                id
            };

        sqlx::query("INSERT INTO anime_tags(anime_id, tag_id) VALUES (?, ?)")
            .bind(anime_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;

    // inserting characters 
    for character in &anime.characters{
        let char_name = &character.name;
        let role = &character.role;
        let character_id = if let Some(id) = character_cache.get(char_name) {
                *id
            } else {
                sqlx::query("INSERT OR IGNORE INTO characters(name) VALUES (?)")
                    .bind(char_name)
                    .execute(&mut *tx)
                    .await?;
                let id: i64 = sqlx::query_scalar("SELECT id FROM characters WHERE name = ?")
                    .bind(char_name)
                    .fetch_one(&mut *tx)
                    .await?;
                character_cache.insert(char_name.clone(), id);
                id
            };

        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM anime_character WHERE anime_id = ? AND character_id = ?"
        )
        .bind(anime_id)
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?;

        if exists.is_none() {
            sqlx::query("INSERT INTO anime_character(anime_id, character_id, role) VALUES (?,?,?)")
                .bind(anime_id)
                .bind(character_id)
                .bind(role)
                .execute(&mut *tx)
                .await?;
        }
    }

    //inserting recommendations 
    for rec in &anime.recommendations{
        let title_rec = &rec.title;
        sqlx::query("INSERT INTO recommendations(anime_id, recommended_title) VALUES (?,?)")
        .bind(anime_id)
        .bind(title_rec)
        .execute(&mut *tx)
        .await?;
    }

} 
}
    tx.commit().await?;
    Ok(())

}
