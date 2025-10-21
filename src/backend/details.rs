
pub use crate::backend::*;

#[derive(Serialize, Default, Deserialize, PartialEq)]
pub struct RelatedAnime{
    title_romaji: String,
    id: i64,
    picture: String,
    relationType: String,
}

#[derive(Serialize, Default, Deserialize, PartialEq)]
pub struct ReccomendResult{
     id: i32,
    title: String,
    picture: String,
    score: f32,
}


#[get("/details")] 
pub async fn get_details(db: web::Data<Pool<Sqlite>>, query: web::Query<SearchQuery>) -> impl Responder {
    let id = format!("{}", query.query);
    match sqlx::query("SELECT * FROM anime WHERE id = ?
    ").bind(&id).fetch_one(db.as_ref()).await {
        Ok(row) => {
        let title = row.try_get("title_romanji").unwrap_or("Unknown").to_string();
        let format = row.try_get("format").unwrap_or("unknown".to_string());
        let description = row.try_get("description").unwrap_or("unknown".to_string());
        let episodes = row.try_get("episodes").unwrap_or(0);
        let status = row.try_get("status").unwrap_or("Unknown".to_string());
        let anime_season = row.try_get("anime_season").unwrap_or("Unknown").to_string();
        let anime_year = row.try_get("anime_year").unwrap_or(0000);
        let picture:String = row.try_get("picture").unwrap_or_default();
        let duration = row.try_get("duration").unwrap_or(0);
        let score = row.try_get("averageScore").unwrap_or(0.0);
        let trailer_url = row.try_get("trailer_url").unwrap_or("Unknown".to_string());
        
        let (synonyms, studios, tags, reccomendation, related) = tokio::join!(
            get_synonyms(db.as_ref(), &id),
            get_studios(db.as_ref(), &id),
            get_tags(db.as_ref(), &id),
            get_recommendations(db.as_ref(), &id),
            get_related(db.as_ref(), &id),
        );

        let anime_deatils = FullAnimeResult{
            title_romanji: title,
            format: format,
            description: description,
            episodes: episodes,
            status:status,
            anime_season:anime_season,
            anime_year:anime_year ,
            picture:picture ,
            duration:duration ,
            score:score ,
            studio: Some(studios),
            synonyms: Some(synonyms),
            tags: Some(tags),
            trailer_url: trailer_url,
            recommendations: reccomendation,
            related_anime: related
        };
        web::Json(anime_deatils)
        }
        Err(_) => {
            web::Json(FullAnimeResult::default())
        }
    }

}

pub async fn get_synonyms(db: &Pool<Sqlite>, id: &String) -> Vec<String> {
    let r = match sqlx::query("SELECT s.synonym FROM synonyms s WHERE s.anime_id = ?")
    .bind(id).fetch_all(db).await {
        Ok(vecs) => vecs,
        Err(e)=>{
            dbg!(e);
            return vec![];
        }
    };
    let mut all_syn = vec![];
    for row in r{
        match row.try_get("synonym") {
            Ok(syn) => all_syn.push(syn),
            Err(e) => {
                dbg!(e); // debug the error and continue addiing other sysnonyms
            }
        }
    }
    return all_syn;
}

pub async fn get_studios(db: &Pool<Sqlite>, id: &String) -> Vec<String> {
    let r = match sqlx::query("SELECT s.name 
        FROM studios s
        JOIN anime_studio ast ON s.id = ast.studio_id
        WHERE ast.anime_id = ?").bind(id).fetch_all(db).await {
            Ok(vecs) => vecs, 
            Err(e) => {
                dbg!(e);
                return vec![];
            }
        };

    let mut all_stud = vec![];
    for row in r{
        match row.try_get("name") {
            Ok(syn) => all_stud.push(syn),
            Err(e) => {
                dbg!(e);
            }
        }
    }
    return all_stud;
}

pub async fn get_tags(db: &Pool<Sqlite>, id: &String) -> Vec<String> {
    let r = match sqlx::query("SELECT t.tag
        FROM tags t
        JOIN anime_tags at ON t.id = at.tag_id
        WHERE at.anime_id = ?
        ").bind(id).fetch_all(db).await
        {
            Ok(vecs) => vecs, 
            Err(e) => {
                dbg!(e);
                return vec![];
            }
        };

    let mut all_stud = vec![];
    for row in r{
        match row.try_get("tag") {
            Ok(syn) => all_stud.push(syn),
            Err(e) => {
                dbg!(e);
            }
        }
    }
    return all_stud;
}

pub async fn get_recommendations(db: &Pool<Sqlite>, id: &String) -> Vec<ReccomendResult> {

    let r = match sqlx::query("
    SELECT r.recommended_title ,a.id, a.picture, a.averageScore
    FROM recommendations r
    JOIN anime a ON a.title_romanji = r.recommended_title
    WHERE anime_id = ?")
        .bind(id).fetch_all(db).await {
            Ok(recs) => recs,
            Err(e) => {
                dbg!(e);
                return  vec![];
            }
        };
    let mut recommendations: Vec<ReccomendResult> = vec![];
    for row in r{
        let recommended_result = ReccomendResult{
        title: row.try_get("title_romanji").unwrap_or("Unknown".to_string()),
        id: row.try_get("id").unwrap_or(-1),
        picture: row.try_get("picture").unwrap_or("Unknown".to_string()), // i should have made sure that there are no nulls in pictures adding unknown image where non existant
        score: row.try_get("averageScore").unwrap_or(0.0)
        };
        if !recommendations.contains(&recommended_result) { // if already in list dont add again, need to sanitize the data in the database so this dosent happen
            recommendations.push(recommended_result);
        }
        
       
    }
    recommendations
}

pub async fn get_related(db: &Pool<Sqlite>, id: &String) -> Vec<RelatedAnime> {
    let r = match sqlx::query("
    SELECT r.related_name, r.relation_type, a.id, a.picture 
    FROM related_anime r
    JOIN anime a ON a.title_romanji = r.related_name
    WHERE anime_id = ?")
        .bind(id).fetch_all(db).await {
        Ok(related) => related,
        Err(e)  => {
            dbg!(e);
            return vec![];
        }       
    };
    let mut related:Vec<RelatedAnime> = vec![];
    for row in r {
        let rel = RelatedAnime{
            title_romaji: row.try_get("realted_name").unwrap_or("Unkown".to_string()),
            id: row.try_get("id").unwrap_or(-1),
            picture: row.try_get("picture").unwrap_or("Unkonwn".to_string()),
            relationType: row.try_get("relation_type").unwrap_or("Unknown".to_string())
        };

       if !related.contains(&rel){
        related.push(rel);
       }
    }

    related
}