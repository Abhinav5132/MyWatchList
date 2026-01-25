use std::{collections::HashMap, time::Duration};

use reqwest::Client;

use crate::backend::{AnimeStructs::{Anime, PartialUpdate}};
pub use crate::backend::*;
use crate::backend::initialize::*;

#[derive(Serialize, Deserialize)]
struct UpdatedAtResult{
    data: DataPage2,
}

#[derive(Serialize, Deserialize)]
struct DataPage2{
    pub page: Page2
}

#[derive(Serialize, Deserialize)]
struct Page2 {
    pub media: Anime
}

pub async fn full_update(db: web::Data<Pool<Sqlite>>, title: &String)-> anyhow::Result<()>{
    let mut studio_cache: HashMap<String, i64> = HashMap::new();
    let mut tag_cache: HashMap<String, i64> = HashMap::new(); // this hella inefficient but it is what it is 
    let mut character_cache: HashMap<String, i64> = HashMap::new();

    let anilist_query = 
    "query ($title: String) {
    Media(type: ANIME, search: $title) {
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

    ";

    let client = Client::new();
    let mut tx = db.begin().await?;

    let variables = serde_json::json!({
            "title": title
    });

    let json:UpdatedAtResult  = loop {
        let res = client
            .post("https://graphql.anilist.co")
            .json(&serde_json::json!({ "query": anilist_query, "variables": variables }))
            .send()
            .await?;

        let status = res.status();

        // Check if we got rate limited (429 Too Many Requests)
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            println!("Rate limited! Waiting 45 seconds before retry...");
            tokio::time::sleep(Duration::from_secs(45)).await;
            continue;
        }

        // Check for other HTTP errors
        if !status.is_success() {
            println!("HTTP error {}: Waiting 5 seconds before retry...", status);
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Try to parse the response
        match res.json::<UpdatedAtResult>().await {
            Ok(data) => break data,
            Err(e) => {
                println!(
                    "Failed to parse response: {}. Waiting 5 seconds before retry...",
                    e
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    };

    let entry = json.data.page.media;
    let id = add_anime(&entry, &mut tx).await?;
    // inserting synonyms
    let synonyms = entry.get_synonyms();
    add_synonyms(synonyms, id, &mut tx).await?;

    //inserting studios
    let studios = entry.get_studios();
    add_studios(studios, id, &mut tx, &mut studio_cache).await?;

    //inserting related
    let related = entry.get_related();
    add_related(related, id, &mut tx).await?;

    //inserting tags
    let tags = entry.get_tags();
    add_tags(tags, &mut tag_cache, id, &mut tx).await?;

    //inserting characters
    let characters = entry.get_characters();
    add_characters(characters, &mut character_cache, &mut tx, id).await?;

    //inserting recommendations
    let recommendations = entry.get_recommended();
    add_recommendations(recommendations, id, &mut tx).await?;

    Ok(())
}