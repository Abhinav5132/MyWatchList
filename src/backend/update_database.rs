use actix_web::web::Data;
use anyhow::Result;
use reqwest::Client;
use sqlx::{Pool, Sqlite};
pub use crate::backend::*;
pub use crate::backend::AnimeStructs::Anime;

//TODO: This whole thing is shit

#[derive(Deserialize, Debug)]
pub struct UpdateCurrentResponse {
    pub data: Option<AnilistData>,
}

#[derive(Deserialize, Debug)]
pub struct AnilistData {
    pub Media: Option<AnimeMedia>,
}

#[derive(Deserialize, Debug)]
pub struct AnimeMedia {
    pub updatedAt: Option<i64>,
    pub popularity: Option<i64>,
}

/*updates the anime that are already in the db and are finished can be run way more periodically than the rest */
//TODO this needs to also update its reccommendations and related as those can change.
pub async fn update_already_in_db(db: web::Data<Pool<Sqlite>>)->anyhow::Result<()> { 
    let updatedAt:i64 = sqlx::query("SELECT updatedAt 
    FROM anime 
    ORDER BY updatedAt ASC 
    LIMIT 1")
    .fetch_one(db.as_ref()).await?
    .try_get("updatedAt")?;

    let current_anime_list = sqlx::query("SELECT title_romanji FROM anime WHERE status = ? ORDER BY updatedAt DESC").bind("FINISHED")
    .fetch_all(db.as_ref()).await?;

    let anilist_query = "
    query ($search: String) {
        Media(search: $search, type: ANIME, sort: [UPDATED_AT_DESC], status: FINISHED) {
            updatedAt
            popularity
        }
    }";

    let client = Client::new();
    let mut tx = db.begin().await?;

    for row in current_anime_list {
        let current_title:String = row.try_get("title_romanji")?;

        let variables = serde_json::json!({
            "search":current_title
        });

        let res = client
            .post("https://graphql.anilist.co")
            .json(&serde_json::json!({ "query": anilist_query, "variables": variables }))
            .send()
            .await?;

        let response:UpdateCurrentResponse = res.json().await?;

        let updated_at_anilist = if let Some(ref data) = response.data{
            if let Some(media) = &data.Media{
                media.updatedAt.unwrap_or(-1)
            } 
            else {-1}
        } else {-1};

        let popularity_anilist = if let Some(ref data) = response.data{
            if let Some(media) = &data.Media{
                media.popularity.unwrap_or(-1)
            } 
            else {-1}
        } else {-1};

        if updatedAt >= updated_at_anilist {
            break; // we are up to date
        }

        sqlx::query("UPDATE anime SET popularity = ?, updatedAt = ?")
        .bind(popularity_anilist)
        .bind(updatedAt).execute(&mut *tx).await?;

    }

    tx.commit().await?;
    Ok(())
}

/*updates anime that are releasing*/
pub async fn update_ones_not_in_db(db: Data<Pool<Sqlite>>) -> Result<()> {
    let updatedAt:i64 = sqlx::query("SELECT updatedAt 
    FROM anime 
    ORDER BY updatedAt ASC 
    LIMIT 1")
    .fetch_one(db.as_ref()).await?
    .try_get("updatedAt")?;

    let anilist_query = "
        query ($page: Int, $perPage: Int) {
            Page(page: $page, perPage: $perPage) {
                media(type: ANIME, sort: [UPDATED_AT_DESC], status_not: FINISHED) {
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
                            type
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
                        voiceActors {
                            name {
                                full
                            }
                        }
                    }
                }
                trailer {
                    site
                    id
                }
                recommendations(perPage: 10, sort: [RATING_DESC]) {
                    nodes {
                        media {
                            title {
                                romaji
                            }
                        }
                        rating
                    }
                }

                airingSchedule(notYetAired: true, perPage: 1) {
                    nodes {
                        episode
                        airingAt
                    }
                }
                }
            }
            }

    
    ";

    Ok(())
}

