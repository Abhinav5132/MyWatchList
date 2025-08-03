#![allow(non_snake_case)]
use std::{fs::File};
use std::io::Read;
use sqlx::Sqlite;
use serde::{Deserialize, Serialize};
use sqlx::Pool;

#[derive(Debug, Deserialize, Serialize)]
pub struct AnimeEntry {
    id: u64,
    titleEnglish: Option<String>,
    titleRomaji: String,
    description: Option<String>,
    format: Option<String>,
    episodes: Option<u32>,
    status: Option<String>,
    startDate: Option<String>,
    endDate: Option<String>,
    season: Option<String>,
    seasonYear: Option<u32>,
    thumbnailImage: Option<String>,
    coverImage: Option<String>,
    duration: Option<u32>,  // in minutes
    popularity: Option<u64>,
    averageScore: Option<u32>,
    synonyms: Vec<String>,
    tags: Vec<String>,
    genres: Vec<String>,
    studios: Vec<String>,
    relations: Vec<RelatedAnime>,
    trailer: Option<String>,
    characters: Vec<Characters>,
    recommendations: Vec<Recommendation>,
    bannerImage: Option<String>,
    nextAiringEpisode: Option<NextAiringEpisode>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Relations {
    nodes: Vec<RelatedAnime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RelatedAnime {
    title: String,
    r#type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)] 
pub struct Characters{
    name: Option<String>,
    role: Option<String>,
    image: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Recommendation{
    title: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct  NextAiringEpisode{
    episode: Option<i32>,
    airingAt: Option<i64>,
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
    let title_romanji = &anime.titleRomaji;
    let title_english = anime.titleEnglish.as_deref()
        .filter(|title| !title.is_empty())
        .unwrap_or(&anime.titleRomaji);
    let description = anime.description.as_deref().unwrap_or("I dont know man, google it.");
    let format = anime.format.as_deref().unwrap_or("Unknown");
    let episodes = anime.episodes.unwrap_or(0);
    let status = anime.status.as_deref().unwrap_or("Unknown");
    let anime_season = anime.season.as_deref().unwrap_or("Unknown");
    let anime_year = anime.seasonYear.unwrap_or(0);
    let start_date = anime.startDate.as_deref().unwrap_or("Unknown");
    let end_date = anime.endDate.as_deref().unwrap_or("Unknown");
    let duration = anime.duration.unwrap_or(0); 
    let score = anime.averageScore.unwrap_or(0);
    let popularity = anime.popularity.unwrap_or(0) as i64;
    let thumbnail = anime.thumbnailImage.as_deref().unwrap_or("none");
    let picture = anime.coverImage.as_deref().unwrap_or("none");
    let trailerUrl = anime.trailer.as_deref().unwrap_or("none");
    let banner_image = anime.bannerImage.as_deref().unwrap_or("none");
    let next_episode = anime.nextAiringEpisode.as_ref().and_then(|ep| ep.episode).unwrap_or(-1);
    let next_episode_date = anime.nextAiringEpisode.as_ref().and_then(|time| time.airingAt).unwrap_or(-1);
    let anime_id = sqlx::query(
        "INSERT INTO anime
        (title_english, title_romanji, description, format, episodes, status, start_date, end_date, anime_season, 
        anime_year, thumbnail, picture, duration, averageScore, popularity, banner_image, trailer_url, next_episode, next_episode_airing_at) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(title_english)
    .bind(title_romanji)
    .bind(description)
    .bind(format)
    .bind(episodes)
    .bind(status)
    .bind(start_date)
    .bind(end_date)
    .bind(anime_season)
    .bind(anime_year)
    .bind(thumbnail)
    .bind(picture)
    .bind(duration)
    .bind(score)
    .bind(popularity)
    .bind(banner_image)
    .bind(trailerUrl)
    .bind(next_episode)
    .bind(next_episode_date)
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
        .bind(&related_anime.title)
        .bind(&related_anime.r#type.as_deref().unwrap_or(""))
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
        let char_name = character.name.as_deref().unwrap_or("");
        let role = &character.role;
        let img = character.image.as_deref().unwrap_or("none");
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
                character_cache.insert(char_name.to_string().clone(), id);
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
            sqlx::query("INSERT INTO anime_character(anime_id, character_id, role, image) VALUES (?,?,?,?)")
                .bind(anime_id)
                .bind(character_id)
                .bind(role)
                .bind(img)
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
