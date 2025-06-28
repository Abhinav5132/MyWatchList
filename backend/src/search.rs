use crate::*;

#[get("/search")]
pub async fn main_search(db: web::Data<Pool<Sqlite>>, query: web::Query<SearchQuery>) -> impl Responder {    
    let title = format!("%{}%", query.query);
    match sqlx::query("
            SELECT anime.title, anime.picture, anime.id
            FROM anime
            WHERE anime.title LIKE ? COLLATE NOCASE
            ORDER BY anime.popularity DESC 
            LIMIT 10"
        )
        .bind(&title)
        .fetch_all(db.get_ref())
        .await
    {
        Ok(rows) => {
            let names: Vec<AnimeResult> = rows.into_iter()
            .map(|r| AnimeResult{
                id: r.try_get("id").unwrap_or_default(),
                title: r.try_get("title").unwrap_or_default(),
                picture: r.try_get("picture").ok(),
            })
            .collect();
            if names.is_empty() {
                // this later needs to fetch everything in the anime table using this anime id so we can create anime structs
                match sqlx::query(
                    "
                SELECT title, picture, id FROM anime 
                JOIN synonyms ON anime.id = synonyms.anime_id
                WHERE synonyms.synonym LIKE ? LIMIT 10
                ",
                )
                .bind(&title)
                .fetch_all(db.as_ref())
                .await
                {
                    Ok(rows) => {
                        let names: Vec<AnimeResult> = rows.into_iter().map(|r| AnimeResult{
                            id: r.try_get("id").unwrap_or_default(),
                            title: r.try_get("title").unwrap_or_default(),
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
