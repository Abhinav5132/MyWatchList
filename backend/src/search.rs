use std::vec;

use crate::*;

#[derive(Deserialize)]
struct SearchQueryPage {
    query: String,
    page: Option<u32>,
}

#[derive(Serialize,Deserialize)]
struct ScrollingResults{
    id: i32,
    title_english: String,
    title_romanji: String,
    banner_image: String,
    averageScore: u32,
    description: String,
    start_date: String,
    duration: u32,
    format: String
}

#[derive(Serialize,Deserialize)]
struct TrendingResults{
    id: i32,
    title_english: String,
    title_romanji: String,
    thumbnail: String,
    averageScore: u32

}

#[derive(Serialize,Deserialize)]
struct TrendingResponse {
    new_popular: Vec<TrendingResults>,
    most_popular: Vec<TrendingResults>,
    scroll_popular: Vec<ScrollingResults>,
}

#[get("/search")]
pub async fn main_search(db: web::Data<Pool<Sqlite>>, query: web::Query<SearchQueryPage>) -> impl Responder {    
    let title = format!("%{}%", query.query);
    let page = query.page.unwrap_or_default();
    let offset = (page - 1) * 28;
    match sqlx::query("
            SELECT anime.title_romanji, anime.picture, anime.id
            FROM anime
            WHERE anime.title_romanji LIKE ? COLLATE NOCASE
            ORDER BY anime.popularity DESC 
            LIMIT 28 OFFSET ?"
        )
        .bind(&title)
        .bind(offset)
        .fetch_all(db.get_ref())
        .await
    {
        Ok(rows) => {
            let names: Vec<AnimeResult> = rows.into_iter()
            .map(|r| AnimeResult{
                id: r.try_get("id").unwrap_or_default(),
                title: r.try_get("title_romanji").unwrap_or_default(),
                picture: r.try_get("picture").ok(),
            })
            .collect();
            if names.is_empty() {
                match sqlx::query(
                    "
                SELECT title, thumbnail, id FROM anime 
                JOIN synonyms ON anime.id = synonyms.anime_id
                WHERE synonyms.synonym LIKE ? LIMIT 28 OFFSET ?
                ",
                )
                .bind(&title)
                .bind(offset)
                .fetch_all(db.as_ref())
                .await
                {
                    Ok(rows) => {
                        let names: Vec<AnimeResult> = rows.into_iter().map(|r| AnimeResult{
                            id: r.try_get("id").unwrap_or_default(),
                            title: r.try_get("title_romanji").unwrap_or_default(),
                            picture: r.try_get("picture").ok(),
                    }).collect();
                        
                        return web::Json(names);
                    }
                    Err(_) =>  {return web::Json(vec![AnimeResult{
                        id: -1,
                        title: "Unable to query the database".into(),
                        picture: None,
                    }])}
                }
            }
            web::Json(names)
        }
        Err(_) => web::Json(vec![AnimeResult{
                        id: -1,
                        title: "Unable to query the database".into(),
                        picture: None,
                    }]),
    }
}

#[get("/trending")]
pub async fn trending_search(db: web::Data<Pool<Sqlite>>) -> impl Responder { 
    //return new anime based on startdate 
    let mut new_popular: Vec<TrendingResults> = vec![];
    let mut most_popular: Vec<TrendingResults> = vec![];
    let mut scrolling_popular:Vec<ScrollingResults> = vec![];
    match sqlx::query(
        "SELECT title_english, title_romanji, banner_image, id, averageScore, duration, description, format
        FROM anime
        WHERE start_date IS NOT NULL
        AND end_date IS NOT NULL
        AND start_date != 'Unknown' 
        AND start_date >= DATE('now', '-4 months')
        AND (status = 'RELEASING' OR status = 'FINSIHED')
        AND averageScore != 0 AND averageScore != -1
        AND banner_image != 'none'
        AND (format = 'TV' OR format = 'MOVIE')
        ORDER BY popularity DESC, averageScore DESC LIMIT 8;"
    ).fetch_all(db.as_ref()).await {
        Ok(rows)=>{
                scrolling_popular = rows.into_iter().map(|row| ScrollingResults{
                id: row.try_get("id").unwrap_or(-1),
                title_english: row.try_get("title_english").unwrap_or("Unknown".to_string()),
                title_romanji: row.try_get("title_romanji").unwrap_or("Unknown".to_string()),
                banner_image: row.try_get("banner_image").unwrap_or("None".to_string()),
                averageScore: row.try_get("averageScore").unwrap_or(0),
                description: row.try_get("description").unwrap_or("IDK".to_string()),
                format: row.try_get("format").unwrap_or("Unknown".to_string()),
                start_date: row.try_get("start_date").unwrap_or("Unknown".to_string()),
                duration: row.try_get("duration").unwrap_or(0)

            }).collect();

        }   
        Err(e) =>{
            println!("Unable to fetch new trending results results {e}")
        }
    }

    match sqlx::query(
        "SELECT title_english, title_romanji, thumbnail, id, averageScore FROM anime
        WHERE start_date IS NOT NULL
        AND end_date IS NOT NULL
        AND start_date != 'Unknown' 
        AND start_date >= DATE('now', '-4 months')
        AND (status = 'RELEASING' OR status = 'FINSIHED')
        AND averageScore != 0 AND averageScore != -1
        AND banner_image != 'none'
        AND (format = 'TV' OR format = 'MOVIE')
        ORDER BY popularity DESC, averageScore DESC LIMIT 20 OFFSET 5;"
    ).fetch_all(db.as_ref()).await {
        Ok(rows)=>{
                new_popular = rows.into_iter().map(|row| TrendingResults{
                id: row.try_get("id").unwrap_or(-1),
                title_english: row.try_get("title_english").unwrap_or("Unknown".to_string()),
                title_romanji: row.try_get("title_romanji").unwrap_or("Unknown".to_string()),
                thumbnail: row.try_get("thumbnail").unwrap_or("None".to_string()),
                averageScore: row.try_get("averageScore").unwrap_or(0)
            }).collect();

        }   
        Err(e) =>{
            println!("Unable to fetch new trending results results {e}")
        }
    }

    // return most trending anime based on their popularity stats
    match sqlx::query(
        "SELECT title_romanji, title_english, thumbnail, id, averageScore
        FROM anime
        ORDER BY popularity DESC LIMIT 20;"
    ).fetch_all(db.as_ref()).await {
        Ok(rows)=> {
            most_popular = rows.into_iter().map(|row| TrendingResults{
                id: row.try_get("id").unwrap_or(-1),
                title_english: row.try_get("title_english").unwrap_or("Unknown".to_string()),
                title_romanji: row.try_get("title_romanji").unwrap_or("Unknown".to_string()),
                thumbnail: row.try_get("thumbnail").unwrap_or("None".to_string()),
                averageScore: row.try_get("averageScore").unwrap_or(0)
            }).collect();
        }
        Err(_) => {
            println!("Unable to fetch most trending results results")
        }
    }

    web::Json(TrendingResponse{
        new_popular: new_popular,
        most_popular: most_popular,
        scroll_popular: scrolling_popular
    })

    //return recommendations based on stuff in their watch list, tags from stuff in their watch list

    //return recommendations from friends

}