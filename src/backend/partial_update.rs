use std::{collections::{HashMap, HashSet}, time::Duration};

use reqwest::Client;
use serde_json::json;

use crate::backend::{AnimeStructs::{Date, NextAiringEpisode, PartialUpdate, Recommendations, Relations}, initialize::{add_recommendations, add_related}};
pub use crate::backend::*;

#[derive(Serialize, Deserialize)]
struct PartialAliasedResult{
    pub data: HashMap<String, PartialMedia>,
}

#[derive(Serialize, Deserialize)]
struct PartialMedia {
    pub nextAiringEpisode: Option<NextAiringEpisode>,
    pub updatedAt: Option<i64>,
    pub episodes: Option<u32>,
    pub status: Option<String>,
    pub endDate: Option<Date>,
    pub popularity: Option<u64>,
    pub averageScore: Option<u32>,
    pub relations: Option<Relations>,
    pub recommendations: Option<Recommendations>,
}

impl PartialUpdate for PartialMedia {
    fn updated_at(&self) -> Option<i64> {
        self.updatedAt
    }

    fn episodes(&self) -> Option<u32> {
        self.episodes
    }

    fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    fn end_date(&self) -> Option<&Date> {
        self.endDate.as_ref()
    }

    fn popularity(&self) -> Option<u64> {
        self.popularity
    }

    fn average_score(&self) -> Option<u32> {
        self.averageScore
    }

    fn relations(&self) -> Option<&Relations> {
        self.relations.as_ref()
    }

    fn next_airing(&self) -> Option<&NextAiringEpisode> {
        self.nextAiringEpisode.as_ref()
    }

    fn recommendations(&self) -> Option<&Recommendations>{
        self.recommendations.as_ref()
    }
}

fn build_query_and_variables(titles:HashMap<i64, String>) -> (serde_json::Value, String, HashMap<String, i64>)
{
    let mut length = 1;
    let mut alias_map = HashMap::new();
    let mut vars = serde_json::Map::new();  
    let starting_query = 
    "
    episodes
    status
    popularity
    averageScore

    endDate {
      year
      month
      day
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
    updatedAt
}";
    let len = titles.len();
    let mut anilist_query = "query (".to_string();
    let mut anilist_query_second = "".to_string();
    for (anime_id, title) in titles.clone() {
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
        let alias = format!("a{}", length);
        alias_map.insert(alias, anime_id);

        length += 1; 

    }

    anilist_query.push_str(&anilist_query_second);
    anilist_query.push('}');

    let variables = serde_json::Value::Object(vars);

    (variables, anilist_query, alias_map)

}

pub async fn partial_update(tx: &mut Transaction<'_, Sqlite>, titles: HashMap<i64, String>)-> anyhow::Result<()>{
    // this needs to update all the anime that are already in the db. 
    // Most quantities will not change so we only need to change the ones that will change 
    if titles.is_empty(){
        return Ok(());
    }
    let (variables , anilist_query, alias_map) = build_query_and_variables(titles);
    let _ = debug_to_file(anilist_query.clone());

    let client = Client::new();

    let json:PartialAliasedResult  = loop {
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
            println!("HTTP error {}: Waiting 5 seconds before retry... partial", status);
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Try to parse the response
        match res.json::<PartialAliasedResult>().await {
            Ok(data) => break data,
            Err(e) => {
                dbg!(
                    "Failed to parse response: {}. Waiting 5 seconds before retry(partial)",
                    e
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    };

    for (alias, media) in json.data{
        let anime_id = alias_map[&alias];
        let episodes = media.get_episodes();
        let status = media.get_status();
        let end_date = media.get_end_date();
        let averageScore = media.get_average_score();
        let popularity = media.get_popularity();
        let (next_episode, next_airing_episode_at) = media.get_airing_at();
        let updatedAt = media.get_updated_at();

        let result = sqlx::query("
    
        UPDATE anime
        SET
        episodes = ?,
        status = ?,
        end_date = ?,
        averageScore = ?,
        popularity = ?,
        next_episode = ?,
        next_episode_airing_at = ?,
        updatedAt = ?
        WHERE id = ?;")
        .bind(episodes).bind(status).bind(end_date).bind(averageScore).bind(popularity).bind(next_episode)
        .bind(next_airing_episode_at).bind(updatedAt).bind(anime_id).execute(&mut **tx).await?;

        let related = media.get_related();
        add_related(related, anime_id, tx).await?;

        let recommendations = media.get_recommended();
        add_recommendations(recommendations, anime_id, tx).await?;

    }
    
    
Ok(())

}
