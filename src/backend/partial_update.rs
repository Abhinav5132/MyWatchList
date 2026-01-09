use std::time::Duration;

use reqwest::Client;

use crate::backend::AnimeStructs::{Date, NextAiringEpisode, PartialUpdate, Recommendations, Relations};
pub use crate::backend::*;

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
    pub media: BasicResponse
}

#[derive(Serialize, Deserialize)]
struct BasicResponse {
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

impl PartialUpdate for BasicResponse {
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


pub async fn partial_update(db: web::Data<Pool<Sqlite>>, title: String, id: i64)-> anyhow::Result<()>{
    // this needs to update all the anime that are already in the db. 
    // Most quantities will not change so we only need to change the ones that will change 

    let anilist_query = 
    "query ($title: String) {
  Media(
    type: ANIME
    search: $title
  ) {
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

    let media = json.data.page.media;
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
    WHERE title_romanji = ?;")
    .bind(episodes).bind(status).bind(end_date).bind(averageScore).bind(popularity).bind(next_episode)
    .bind(next_airing_episode_at).bind(updatedAt).bind(title).execute(&mut *tx).await;

    let related = media.get_related();
    for (name, relation) in related {
        sqlx::query("INSERT INTO related_anime(anime_id, related_name, relation_type) VALUES (?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(relation)
            .execute(&mut *tx)
            .await?;
    }

    let recommendations = media.get_recommended();
    for name in recommendations {
        sqlx::query(
            "INSERT INTO recommendations(anime_id, recommended_title) VALUES (?,?)",
        )
        .bind(id)
        .bind(name)
        .execute(&mut *tx)
        .await?;
    }
Ok(())

}
