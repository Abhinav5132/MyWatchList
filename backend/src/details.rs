pub use crate::*;

#[get("/details")] 
pub async fn get_details(db: web::Data<Pool<Sqlite>>, query: web::Query<SearchQuery>) -> impl Responder {
    let id = format!("{}", query.query);
    match sqlx::query("SELECT * FROM anime WHERE id = ?
    ").bind(id.clone()).fetch_one(db.as_ref()).await {
        Ok(row) => {
        let title = row.try_get("title").unwrap_or("Unknown").to_string();
        let format = row.try_get("format").unwrap_or("unknown".to_string());
        let episodes = row.try_get("episodes").unwrap_or(0);
        let status = row.try_get("status").unwrap_or("Unknown".to_string());
        let anime_season = row.try_get("anime_season").unwrap_or("Unknown").to_string();
        let anime_year = row.try_get("anime_year").unwrap_or(0000);
        let picture:String = row.try_get("picture").unwrap_or_default();
        let duration = row.try_get("duration").unwrap_or(0);
        let score = row.try_get("score").unwrap_or(0.0);

        //synonyms
        let mut r = sqlx::query("SELECT s.synonym FROM synonyms s WHERE s.anime_id = ?")
        .bind(id.clone()).fetch_all(db.as_ref()).await.unwrap_or_default();
        let synonyms = r.into_iter().filter_map(|s| s.try_get("synonym").ok()).collect();
        
        // studios
        r = sqlx::query("SELECT s.name 
                 FROM studios s
                 JOIN anime_studio ast ON s.id = ast.studio_id
                 WHERE ast.anime_id = ?").bind(id.clone()).fetch_all(db.as_ref()).await.unwrap_or_default();

        let studios = r.into_iter().filter_map(|s| s.try_get("name").ok()).collect();
        
        // tags
        r = sqlx::query("SELECT t.tag
                 FROM tags t
                 JOIN anime_tags at ON t.id = at.tag_id
                 WHERE at.anime_id = ?
        ").bind(id.clone()).fetch_all(db.as_ref()).await.unwrap_or_default();
        let tags = r.into_iter().filter_map(|t| t.try_get("tag").ok()).collect();

        let anime_deatils = FullAnimeResult{
            title: title,
            format: format,
            episodes: episodes,
            status:status,
            anime_season:anime_season,
            anime_year:anime_year ,
            picture:picture ,
            duration:duration ,
            score:score ,
            studio: studios,
            synonyms: synonyms,
            tags: tags,
        };
        web::Json(anime_deatils)
        }
        Err(_) => {
            web::Json(FullAnimeResult::default())
        }
    }

}