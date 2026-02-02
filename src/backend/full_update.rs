use std::{collections::{HashMap, HashSet}, fs::File, io::Write, time::Duration};

use reqwest::Client;
use serde_json::json;

use crate::backend::AnimeStructs::{Anime, PartialUpdate, Tag};
pub use crate::backend::*;
use crate::backend::initialize::*;

#[derive(Serialize, Deserialize)]
struct FullAliasedResult{
    data: HashMap<String, Anime>
}

fn build_query_and_variables(titles: HashSet<String>) -> (serde_json::Value, String){
    let starting_query = "
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
    }";
    let mut length = 0;
    let mut vars = serde_json::Map::new();  
    let len = titles.len();
    let mut anilist_query = "query (".to_string();
    let mut anilist_query_second = "".to_string();
    for title in titles.clone() {
        vars.insert(format!("t{}", length), json!(title));
        if length != len{
            anilist_query.push_str(format!("$t{}: String!, ", length).as_str());
        }
        else {
            anilist_query.push_str(format!("$t{}: String!) {{", length).as_str());

        }
        let mut aliased_query = format!("a{}: Media(type: ANIME, search: $t{}) {{", length, length);
        aliased_query.push_str(starting_query);
        anilist_query_second.push_str(&aliased_query); 
        length += 1; 
    }
    anilist_query.push_str(&anilist_query_second);
    anilist_query.push('}');

    let variables = serde_json::Value::Object(vars);
    (variables, anilist_query)
}
pub fn debug_to_file(debug: String) -> std::io::Result<()> {
    let mut file = File::create("debug.txt")?;
    file.write_all(debug.as_bytes())?;
    Ok(())
}
pub async fn full_update(
tx: &mut Transaction<'_, Sqlite>, 
titles: HashSet<String>, 
studio_cache: &mut HashMap<String, i64>,
tag_cache: &mut HashMap<String, i64>,
character_cache: &mut HashMap<String, i64>
)-> anyhow::Result<()>{

    if titles.is_empty(){
       return Ok(());
    }
    let (variables, anilist_query) = build_query_and_variables(titles); 

    let _ = debug_to_file(anilist_query.clone());
    let client = Client::new();
    let json:FullAliasedResult  = loop {
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
            println!("HTTP error {}: Waiting 5 seconds before retry... full", status);
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Try to parse the response
        match res.json::<FullAliasedResult>().await {
            Ok(data) => break data,
            Err(e) => {
                dbg!(
                    "Failed to parse response: {}. Waiting 5 seconds before retry(full)",
                    e
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    };

    let entry = json.data;
    // TODO CHANGE tags and studios to also try fetch the ID from DB IF NOT in cache.
    for (_, anime) in entry{
        let id = match add_anime(&anime, tx).await{
            Ok(id) => {
                id
            },
            Err(e) => {
                dbg!(&e);
                return Err(e);
            }
        };

        if id == 0 {
            return Ok(());
        }
        // inserting synonyms
        let synonyms = anime.get_synonyms();
        add_synonyms(synonyms, id, tx).await?;

        //inserting studios
        let studios = anime.get_studios();
        add_studios(studios, id, tx, studio_cache).await?;

        //inserting related
        let related = anime.get_related();
        add_related(related, id, tx).await?;

        //inserting tags
        let tags = anime.get_tags();
        add_tags(tags, tag_cache, id, tx).await?;

        //inserting characters
        let characters = anime.get_characters();
        add_characters(characters, character_cache, tx, id).await?;

        //inserting recommendations
        let recommendations = anime.get_recommended();
        add_recommendations(recommendations, id, tx).await?;

    }
    
    Ok(())
}

async fn add_characters(
    characters: Vec<(&str, &str, &str)>, 
    character_cache: &mut HashMap<String, i64>, 
    tx: &mut Transaction<'_, Sqlite>,
    id: i64
) -> anyhow::Result<()>{

    if characters.is_empty(){
        return Ok(());
    }
    // this hash set is needed to verify if characters are ignored becasue they are already in the list then we want to 
    //make sure not to insert the previous row into anime_characters
    let mut inserted_characters = std::collections::HashSet::new();
    for (name, role, image) in characters {
        let character_id = 
        if let Some(id) = character_cache.get(name) {
            *id
        } else {

            let mut id:i64 = match sqlx::query_scalar("SELECT id from characters WHERE name = ?")
            .bind(name).fetch_one(&mut **tx).await {
                Ok(id) => {
                    id
                }
                Err(e) => {
                    dbg!(e);
                    0i64
                }
            };

            if id == 0{
                id = sqlx::query("INSERT OR IGNORE INTO characters(name) VALUES (?)")
                .bind(name)
                .execute(&mut **tx)
                .await?
                .last_insert_rowid();  
            } 
            
            character_cache.insert(name.to_string(), id);
            id

            
        };
        if inserted_characters.insert(character_id){
            sqlx::query("INSERT INTO anime_character(anime_id, character_id, role, image) VALUES (?,?,?,?)")
            .bind(id)
            .bind(character_id)
            .bind(role)
            .bind(image)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

async fn add_tags(tags: &[Tag], tag_cache: &mut HashMap<String, i64>, id: i64, tx: &mut Transaction<'_, Sqlite>) -> anyhow::Result<()>{

    if tags.is_empty(){
        return Ok(());
    }
    for tag in tags {
        let tag_name = tag.name.as_deref().unwrap_or("UNKNOWN");
        let tag_id = if let Some(id) = tag_cache.get(tag_name) {
            *id
        } else {

            let mut id = match sqlx::query_scalar("SELECT id FROM tags WHERE tag = ?").fetch_one(&mut **tx).await {
                Ok(id) => {
                    id
                }
                Err(e) => {
                    dbg!(e);
                    0i64
                }
            };

            if id == 0 {
                id = sqlx::query(
                "INSERT OR IGNORE INTO tags (tag, rank, isAdult) VALUES (?, ? , ?)",)
                .bind(tag_name)
                .bind(tag.rank)
                .bind(tag.isAdult)
                .execute(&mut **tx)
                .await?
                .last_insert_rowid();
            }
            tag_cache.insert(tag_name.to_string(), id);
            id
        };
        sqlx::query("INSERT INTO anime_tags(anime_id, tag_id) VALUES (?, ?)")
            .bind(id)
            .bind(tag_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

async fn add_studio(
    studios: Vec<String>, 
    id: i64, 
    tx: &mut Transaction<'_, Sqlite>, 
    studio_cache: &mut HashMap<String, i64>) -> anyhow::Result<()>
{

    if studios.is_empty(){
        return Ok(());
    }

    let mut inserted_studios = std::collections::HashSet::new();
    for studio in studios {
        let studio_id = if let Some(id) = studio_cache.get(&studio) {
            *id
        } else {

            let mut id = match sqlx::query_scalar("SELECT id FROM studios WHERE name = ?").fetch_one(&mut **tx).await{
                Ok(id) => {
                    id
                }
                Err(e) => {
                    dbg!(e);
                    0
                }
            };

            if id == 0 {
                id = sqlx::query("INSERT OR IGNORE INTO studios (name) VALUES (?)")
                .bind(&studio)
                .execute(&mut **tx)
                .await?
                .last_insert_rowid();
            }
            studio_cache.insert(studio.clone(), id);
            id
        };
        
        if inserted_studios.insert(studio_id){
            match sqlx::query("INSERT INTO anime_studio(anime_id, studio_id) VALUES (?, ?)")
            .bind(id)
            .bind(studio_id)
            .execute(&mut **tx)
            .await{
                Ok(_) => {},
                Err(e) => {
                    dbg!(e);
                }
            }
        }
    }

    Ok(())
}
