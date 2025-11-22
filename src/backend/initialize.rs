#![allow(non_snake_case)]
use std::time::Duration;

use actix_web::web::Data;
use anyhow::Result;
use reqwest::Client;
use sqlx::Sqlite;
use serde::Deserialize;
use sqlx::Pool;

use crate::backend::AnimeStructs::Anime;
pub use crate::backend::*;


#[derive(Deserialize)]
pub struct AnilistResponse {
    pub data: DataPage,
}

#[derive(Deserialize)]
pub struct DataPage {
    pub Page: Page,
}

#[derive(Deserialize)]
pub struct Page {
    pub media: Vec<Anime>,
}

pub async fn initialize_database(db: Data<Pool<Sqlite>>) -> Result<()> {
    println!("INITIALIZING");    
    /*first lets update finished anime from the last updated date. */
    let anilist_query = "
        query ($page: Int, $perPage: Int) {
        Page(page: $page, perPage: $perPage) {
            media(type: ANIME, sort: POPULARITY_DESC) {
            title{
                romaji
                english
            }
            updatedAt
            description
            format
            episodes
            status
            season
            seasonYear
            startDate {
                year
                month
                day
            }
            endDate {
                year
                month
                day
            }
            coverImage {
                extraLarge
                large
                medium
            }
            bannerImage
            duration
            popularity
            averageScore
            synonyms
            genres
            tags {
                name
                rank
                isAdult
            }
            studios {
                nodes {
                    name
                }
            }
            relations {
                edges {
                    relationType
                    node {
                        title {
                            romaji
                        }
                    }
                }
            }
            characters(perPage: 10, sort: [ROLE, RELEVANCE]) {
                edges {
                    role
                    node {
                        name {
                            full
                        }
                        image {
                            medium
                        }
                    }
                }
            }
            recommendations(perPage: 10, sort: [RATING_DESC]) {
                nodes {
                    mediaRecommendation {
                        title {
                            romaji
                        }
                    }
                }
            }

                nextAiringEpisode {
                airingAt
                episode
            }
            }
        }
        }

    ";
    let mut page = 1;
    let per_page = 50;

    let client = Client::new();


    use std::collections::HashMap;
    let mut studio_cache: HashMap<String, i64> = HashMap::new();
    let mut tag_cache: HashMap<String, i64> = HashMap::new();
    let mut character_cache: HashMap<String, i64> = HashMap::new();
    let mut tx = db.begin().await?;

    loop {
        let variables = serde_json::json!({
            "page": page,
            "perPage": per_page,
        });
        let json: AnilistResponse = loop {
            let res = client
            .post("https://graphql.anilist.co")
            .json(&serde_json::json!({ "query": anilist_query, "variables": variables }))
            .send()
            .await?;
            
            let status = res.status();
            
            // Check if we got rate limited (429 Too Many Requests)
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                println!("Rate limited! Waiting 72 seconds before retry...");
                tokio::time::sleep(Duration::from_secs(72)).await;
                continue;
            }

            // Check for other HTTP errors
            if !status.is_success() {
                println!("HTTP error {}: Waiting 5 seconds before retry...", status);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            // Try to parse the response
            match res.json::<AnilistResponse>().await {
                Ok(data) => break data,
                Err(e) => {
                    println!("Failed to parse response: {}. Waiting 5 seconds before retry...", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }

        };
        
       
        let media_list:Vec<Anime> = json.data.Page.media;

        if media_list.is_empty() {
            dbg!("No more anime to fetch. Database Initialization Complete!");
           break; // No completed anime to update  
        }

        for entry in media_list{
            let updated_at = entry.get_updated_at();

            let title_romaji = entry.get_title_romaji();
            dbg!(title_romaji);
            let title_english = entry.get_title_english();
            let description = &entry.get_description();
            let format = entry.get_format();
            let episodes = entry.get_episodes();
            let status = entry.get_status();
            let start_date = entry.get_start_date();
            let end_date = entry.get_end_date();
            let anime_season = entry.get_season();
            let anime_year = entry.get_season_year();
            let cover = entry.get_cover_images();
            let extra_large = cover.0;
            let large = cover.1;
            let medium = cover.2;
            let duration = entry.get_duration();
            let averageScore = entry.get_averageScore();
            let popularity = entry.get_popularity();
            let banner_image = entry.get_bannner_image();
            let next = entry.get_airing_at();
            let next_episode = next.0;
            let next_episode_airing_at = next.1;

            let is_present: i64 = sqlx::query("SELECT COUNT(1) FROM anime WHERE title_romanji = ?")
            .bind(title_romaji)
            .fetch_one(&mut *tx)
            .await?
            .try_get(0)?;

            if is_present == 1 {
                continue;
            }

            let id = sqlx::query(" 
            INSERT OR IGNORE INTO anime
            (title_english, title_romanji, description, format, episodes, status, start_date, end_date, anime_season, 
            anime_year, extraLargeImage, largeImage, mediumImage, duration, averageScore, popularity, banner_image, next_episode, next_episode_airing_at, updatedAt) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ").bind(title_english).bind(title_romaji).bind(description).bind(format)
            .bind(episodes).bind(status).bind(start_date).bind(end_date)
            .bind(anime_season).bind(anime_year).bind(extra_large).bind(large)
            .bind(medium).bind(duration).bind(averageScore).bind(popularity)
            .bind(banner_image).bind(next_episode).bind(next_episode_airing_at).bind(updated_at)
            .execute(&mut *tx).await?.last_insert_rowid();
 
            // inserting synonyms
            let synonyms = entry.get_synonyms();
             for synonym in synonyms {
                sqlx::query("INSERT INTO synonyms(anime_id, synonym) VALUES (?, ?)")
                .bind(id)
                .bind(synonym)
                .execute(&mut *tx).await?;
            }

            //inserting studios 
            let studios = entry.get_studios(); 
            for studio in studios{
                let studio_id = if let Some(id) = studio_cache.get(&studio) {
                            *id
                        } else {
                            let id = sqlx::query("INSERT OR IGNORE INTO studios (name) VALUES (?)")
                                .bind(&studio)
                                .execute(&mut *tx)
                                .await?.last_insert_rowid();
                            studio_cache.insert(studio.clone(), id);
                            id
                        };
                sqlx::query("INSERT INTO anime_studio(anime_id, studio_id) VALUES (?, ?)")
                    .bind(id)
                    .bind(studio_id)
                    .execute(&mut *tx)
                    .await?;
            }

            let related = entry.get_related();
            for (name, relation) in related {
                sqlx::query("INSERT INTO related_anime(anime_id, related_name, relation_type) VALUES (?, ?, ?)")
                    .bind(id)
                    .bind(name)
                    .bind(relation)
                    .execute(&mut *tx)
                    .await?;
            }

            let tags = entry.get_tags();
            for tag in tags {
                let tag_name = tag.name.as_deref().unwrap_or("UNKNOWN");
                let tag_id = if let Some(id) = tag_cache.get(tag_name) {
                            *id
                    } else {
                        let id = sqlx::query("INSERT OR IGNORE INTO tags (tag, rank, isAdult) VALUES (?, ? , ?)")
                            .bind(tag_name)
                            .bind(tag.rank)
                            .bind(tag.isAdult)
                            .execute(&mut *tx)
                            .await?.last_insert_rowid();
                        tag_cache.insert(tag_name.to_string(), id);
                        id
                    };

                sqlx::query("INSERT INTO anime_tags(anime_id, tag_id) VALUES (?, ?)")
                    .bind(id)
                    .bind(tag_id)
                    .execute(&mut *tx)
                    .await?;
            }

            let mut inserted_characters = std::collections::HashSet::new();
            let characters = entry.get_characters();
            for (name, role, image) in characters{
                let character_id = if let Some(id) = character_cache.get(name) {
                        *id
                    } else {
                        let id = sqlx::query("INSERT OR IGNORE INTO characters(name) VALUES (?)")
                            .bind(name)
                            .execute(&mut *tx)
                            .await?.last_insert_rowid();
                        character_cache.insert(name.to_string().clone(), id);
                        id
                    };
                if inserted_characters.insert(character_id) {
                sqlx::query("INSERT INTO anime_character(anime_id, character_id, role, image) VALUES (?,?,?,?)")
                    .bind(id)
                    .bind(character_id)
                    .bind(role)
                    .bind(image)
                    .execute(&mut *tx)
                    .await?;
                }
            }

            let recommendations = entry.get_recommended();
            for name in recommendations{
                sqlx::query("INSERT INTO recommendations(anime_id, recommended_title) VALUES (?,?)")
                .bind(id)
                .bind(name)
                .execute(&mut *tx)
                .await?;
            }
        }   
        page += 1;
    }    

    tx.commit().await?;
    return Ok(());
}
