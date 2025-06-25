use std::{collections::HashSet, fs::{File, OpenOptions}, io::{Read, Write}};
use reqwest::Client;
use serde_json::{json, Value};

#[tokio::main]
pub async fn popu_main() -> anyhow::Result<()> {
    let client = Client::new();
    let query = r#"
    query ($page: Int, $perPage: Int) {
        Page(page: $page, perPage: $perPage) {
            media(type: ANIME, sort: [POPULARITY_DESC]) {
                id
                title {
                    romaji
                }
                popularity
                coverImage {
                    extraLarge
                }
                description
            }
        }
    }
    "#;

    // Try to resume from existing file
    let mut all_results: Vec<Value> = if let Ok(mut file) = File::open("popularity.json") {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if contents.trim().is_empty() {
            vec![]
        } else {
            serde_json::from_str(&contents)?
        }
    } else {
        vec![]
    };

    // Track existing anime IDs to prevent duplicates
    let existing_ids: HashSet<i64> = all_results
        .iter()
        .filter_map(|entry| entry["id"].as_i64())
        .collect();

    let mut next_page = (all_results.len() / 50) + 1;

    for page in next_page..=5000 {
        println!("Fetching page {page}");

        let vars = json!({
            "page": page,
            "perPage": 50
        });

        let res = client
            .post("https://graphql.anilist.co")
            .json(&json!({
                "query": query,
                "variables": vars
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        if let Some(media) = res["data"]["Page"]["media"].as_array() {
            if media.is_empty() {
                println!("No more results.");
                break;
            }

            let mut new_entries = 0;

            for anime in media {
                if let Some(id) = anime["id"].as_i64() {
                    if !existing_ids.contains(&id) {
                        all_results.push(anime.clone());
                        new_entries += 1;
                    }
                }
            }

            println!("Added {new_entries} new anime.");
        }

        // Save after each page
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("popularity.json")?;
        file.write_all(serde_json::to_string_pretty(&all_results)?.as_bytes())?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    println!(" Done. Total anime fetched: {}", all_results.len());

    Ok(())
}
