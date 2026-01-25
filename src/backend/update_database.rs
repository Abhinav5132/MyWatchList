use std::time::Duration;

pub use crate::backend::AnimeStructs::Anime;
pub use crate::backend::*;
pub use crate::backend::partial_update::*;
pub use crate::backend::full_update::*;

use crate::{backend::{AnimeStructs::{Title}}, try_or};
use reqwest::Client;
use sqlx::{Pool, Sqlite};

#[derive(Serialize, Deserialize)]
struct UpdatedAtResult{
    pub data: DataPage2,
}

#[derive(Serialize, Deserialize)]
pub struct DataPage2{
    pub Page: Page2
}

#[derive(Serialize, Deserialize)]
pub struct Page2 {
    pub media: Vec<BasicResponse>
}

#[derive(Serialize, Deserialize)]
pub struct BasicResponse {
    pub title: Title,
    pub updatedAt: Option<i64>
}


pub async fn update_database(db: web::Data<Pool<Sqlite>>) -> anyhow::Result<()> {
    let last_updated: i64 = match sqlx::query("SELECT updatedAt FROM anime ORDER BY updatedAt DESC LIMIT 1")
    .fetch_one(db.as_ref()).await{
        Ok(res) => try_or!(res.try_get("updatedAt"), Err(anyhow::Error::msg("Failed to serialize db result"))),
        Err(e) => {
            dbg!(e);
            return Err(anyhow::Error::msg("Failed to fetch from the db"));
        }
    };

    let anilist_query = 
    "query ($page: Int, $perPage: Int) {
        Page(page: $page, perPage: $perPage) {
            media(type: ANIME, sort: [UPDATED_AT_DESC]) {
            title{
                romaji
            }
            updatedAt
            }
        }
    }";

    let mut page = 1;
    let per_page = 50;

    let client = Client::new();
    let mut tx = db.begin().await?;
    
    loop {
        let variables = serde_json::json!({
            "page": page,
            "perPage": per_page,
        });
        let json: UpdatedAtResult = loop {
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
                    dbg!(
                        "Failed to parse response: {}. Waiting 5 seconds before retry...",
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        };
        let media = json.data.Page.media;

        if media.is_empty() {
            dbg!("No more anime to fetch. Database Initialization Complete!");
            break Ok(()); // No completed anime to update  
        }

        for entry in media{
            let title =match entry.title.romaji{
                Some(romanji ) => romanji,
                None => continue
            };
            
            dbg!(&title);

            let updated_at = entry.updatedAt.unwrap_or(last_updated);

            if updated_at >= last_updated{
                // this should end the loop
                break;
            }

            let exists: i64 = match sqlx::query_scalar("SELECT id FROM anime WHERE title_romanji = ?")
            .bind(&title).fetch_one(db.as_ref()).await {
                Ok(c) => c,
                Err(e) => {
                    dbg!(e);
                    continue;
                }
            };

            if exists != 0 {
                if let Err(e) = partial_update(db.clone(), &title, exists).await {
                    dbg!(e);
                    dbg!(&title);
                }
            }

            if let Err (e) = full_update(db.clone(), &title).await {
                dbg!(e);
                dbg!(&title);
            }

        }

        page += per_page
    }

}

// this needs two different logics for if something is already present in the databse and for something that is new. 
// first we start of by getting the title of a certain thing, then we check if we have something with that exact title in the db
// if we do then we only update status, end_date, episodes, popularity, next episode, next episode airing at 