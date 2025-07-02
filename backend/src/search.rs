use crate::*;

#[derive(Deserialize)]
struct SearchQueryPage {
    query: String,
    page: Option<u32>,
}

#[get("/search")]
pub async fn main_search(db: web::Data<Pool<Sqlite>>, query: web::Query<SearchQueryPage>) -> impl Responder {    
    let title = format!("%{}%", query.query);
    let page = query.page.unwrap_or_default();
    let offset = (page - 1) * 28;
    match sqlx::query("
            SELECT anime.title, anime.picture, anime.id
            FROM anime
            WHERE anime.title LIKE ? COLLATE NOCASE
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
                title: r.try_get("title").unwrap_or_default(),
                picture: r.try_get("picture").ok(),
            })
            .collect();
            if names.is_empty() {
                match sqlx::query(
                    "
                SELECT title, picture, id FROM anime 
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
