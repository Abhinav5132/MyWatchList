use std::collections::HashMap;
use std::collections::HashSet;
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


pub async fn update_database(db: web::Data<Pool<Sqlite>>, interval: Duration) -> anyhow::Result<()> {
    let mut studio_cache: HashMap<String, i64> = HashMap::new();
    let mut tag_cache: HashMap<String, i64> = HashMap::new(); 
    let mut character_cache: HashMap<String, i64> = HashMap::new();

    loop{
        let last_updated: i64 = match sqlx::query("SELECT updatedAt FROM anime ORDER BY updatedAt DESC LIMIT 1")
    .   fetch_one(db.as_ref()).await{
        Ok(res) => try_or!(res.try_get("updatedAt"), Err(anyhow::Error::msg("Failed to serialize db result"))),
        Err(e) => {
            dbg!(e);
            return Err(anyhow::Error::msg("Failed to fetch from the db"));
        }
        };
        println!("Background task: Checking for anime updates...");
        

        let mut tx = db.begin().await?;

        let (mut partial_update_set, mut full_update_set) = 
        fetch_all_to_update_names(last_updated, db.clone()).await;

        match full_update(&mut tx, full_update_set, &mut studio_cache, &mut tag_cache, &mut character_cache).await{
            Ok(_) => {},
            Err(e) => {
                dbg!(e);
            }
        };
        match partial_update(&mut tx, partial_update_set).await{
            Ok(_) => {},
            Err(e) => {
                dbg!(e);
            }
        }
        tx.commit().await?;

        let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

        sqlx::query("INSERT INTO settings(id, last_updated_at_time) VALUES (1, ?) ON CONFLICT DO UPDATE SET last_updated_at_time = ?")
        .bind(current_time)
        .bind(current_time)
        .execute(db.as_ref()).await?;
        println!("Background task: Finished checking going to sleep");
        sleep(interval).await;
    }
}


pub async fn fetch_all_to_update_names(last_updated: i64, db: web::Data<Pool<Sqlite>>)-> (HashMap<i64, String>, HashSet<String>){
        let mut partial_update_set: HashMap<i64, String> = HashMap::new();
        let mut full_update_set: HashSet<String> = HashSet::new();
        let mut should_stop = false;
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
        
        loop {
            if should_stop {
                break (partial_update_set, full_update_set);
            }
            let variables = serde_json::json!({
                "page": page,
                "perPage": per_page,
            });
            let json: UpdatedAtResult = loop {
                let res = match client
                    .post("https://graphql.anilist.co")
                    .json(&serde_json::json!({ "query": anilist_query, "variables": variables }))
                    .send()
                    .await{
                        Ok(res) => {
                            res
                        }
                        Err(e) => {
                            dbg!(e);
                            break UpdatedAtResult { data: DataPage2 { Page: Page2 { media: vec![] } } };
                        }
                    };

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
            for entry in media{
                let title = match entry.title.romaji{
                    Some(romanji ) => romanji,
                    None => continue
                };
                dbg!(&title);
                let updated_at = match entry.updatedAt {
                    Some(time) => time,
                    None => {
                        dbg!("Skipping entry without updatedAt: {}", &title);
                        continue;
                    }
                };

                if updated_at >= last_updated{
                    // this should end the loop
                    should_stop = true;
                    dbg!("loop_broken");
                    break;
                }
                let exists: i64 = match sqlx::query_scalar("SELECT id FROM anime WHERE title_romanji = ?")
                .bind(&title).fetch_optional(db.as_ref()).await {
                    Ok(c) =>  c.unwrap_or_default(),
                    Err(e) => {
                        dbg!(e);
                        continue;
                    }
                };

                if exists != 0 {
                    partial_update_set.insert(exists, title.clone());
                    continue;
                }

                full_update_set.insert(title.clone());
                continue;

            }
            page += 1
    }
    
}