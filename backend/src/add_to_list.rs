use actix_web::{web::{BufMut, Data, Json}, HttpResponse};
use serde_json::json;

use crate::*;


// watch list is always 1
// recommend is always 2
// best of all time ranking list is always 3

#[derive(Deserialize)]
pub struct AddToList{
    anime_id: i64, 
    list_name: String,
    user_id: i64
}

#[derive(Serialize)]
struct AList{
    name: String,
    id: i32
}

#[derive(Serialize)]
struct AllListSimple{
    list: Vec<AList>
}
#[derive(Serialize)]
struct AllAnimeSimple{
    anime: Vec<AnimeResult>
}

#[derive(Deserialize)]
struct FetchLists{
    user_id: i64 
}

#[derive(Deserialize)]
struct FetchAnimes{
    watch_list_id: i32,
}

#[derive(Deserialize)]
pub struct AddListToUser{

    user_id: i64,
    name: String,
    privacy_type: String,
}

#[derive(Serialize)]
pub struct ExistsInList{
    exists: bool
}

#[post("/add-anime-to-list")]
pub async fn add_anime_to_list(db: web::Data<Pool<Sqlite>>,to_add: Json<AddToList>) ->HttpResponse{
    let anime_id = &to_add.anime_id;
    let list_name = &to_add.list_name;
    let user_id = &to_add.user_id;

    let count:i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM watch_anime_list WHERE watch_name = ? AND anime_id = ? AND user_id = ?"
    )
    .bind(list_name)
    .bind(anime_id).bind(user_id)
    .fetch_one(db.as_ref())
    .await.unwrap_or(0);//add actual error handeling here;
    
    if count < 1 {
        match sqlx::query("INSERT INTO watch_list_anime(watch_name, anime_id, user_id) VALUES (?,?,?);")
        .bind(list_name)
        .bind(anime_id)
        .bind(user_id)
        .execute(db.as_ref()).await {
        Ok(_) => {
            dbg!("Excecuted properly");
            HttpResponse::Ok().into()
            }

        Err(e) => {
            dbg!(e);
            HttpResponse::InternalServerError().into()
            }
        }
    } else {
        HttpResponse::Conflict().body("Anime is already in list")
    }
    
    
}

#[post("/remove-form-list")]
pub async fn remove_from_list(db: web::Data<Pool<Sqlite>>,to_add: Json<AddToList>) ->HttpResponse{
    let anime_id = &to_add.anime_id;
    let list_name = &to_add.list_name;
    let user_id = &to_add.user_id;
    match sqlx::query("
    DELETE FROM watch_list_anime 
    WHERE anime_id = ? AND watch_name = ? AND user_id = ?;")
    .bind(anime_id)
    .bind(list_name)
    .bind(user_id)
    .execute(db.as_ref()).await {
        Ok(_) => {
            dbg!("Excecuted properly");
            HttpResponse::Ok().into()
        }

        Err(e) => {
            dbg!(e);
            HttpResponse::InternalServerError().into()
        }
    }
}

#[post("/add-list-to-user")]
pub async fn create_watch_list(db: Data<Pool<Sqlite>>, to_add: Json<AddListToUser>)-> HttpResponse{
    match create_list(&db, &to_add.name, &to_add.user_id, &to_add.privacy_type).await {
        Ok(_) => {
            HttpResponse::Ok().into()
        }

        Err(_) => {
            HttpResponse::InternalServerError().into()
        }
    }
}

pub async fn create_list(db: &Pool<Sqlite>, name: &String, user_id:&i64, privacy_type: &String)->Result<(), sqlx::Error>{
    sqlx::query("INSERT INTO watch_list(name, user_id, privacy_type) VALUES (?,?,?);")
    .bind(name)
    .bind(user_id)
    .bind(privacy_type)
    .execute(db).await?;

    Ok(())
}


#[post("/remove-list-from-user")]
pub async fn remove_watch_list(db: Data<Pool<Sqlite>>, to_add: Json<AddListToUser>)-> HttpResponse{
    match sqlx::query("DELETE FROM watch_list WHERE name = ? and user_id = ?;")
    .bind(&to_add.name)
    .bind(&to_add.user_id)
    .execute(db.as_ref()).await {
        Ok(_) => {
           return HttpResponse::Ok().into()
        }

        Err(_) => {
           return HttpResponse::InternalServerError().into()
        }
    }
}

#[get("/fetch-all-lists")]
pub async fn fetch_all_lists(db: Data<Pool<Sqlite>>, user: Json<FetchLists>) -> HttpResponse {
    let lists = sqlx::query("SELECT id, name FROM watch_list WHERE user_id = ?")
    .bind(&user.user_id)
    .fetch_all(db.as_ref()).await;
    let mut all_list:Vec<AList> = vec![];
    match lists {
        Ok(row) =>{
            for list in row{
                let id:i32 = match list.try_get("id") {
                    Ok(id)=>id,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                };

                let name:String = match list.try_get("name") {
                    Ok(name) => name,
                    Err(e) => {
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                };

                let alist = AList{
                    id: id,
                    name: name,
                };

                all_list.push(alist);

            }
        }
        Err(_)=>{
            return HttpResponse::InternalServerError().into();
        }
    };

    HttpResponse::Ok().json(json!(AllListSimple{
        list: all_list
    }))
} 

#[get("/get-animes-from-list")]
pub async fn fetch_all_anime_from_list(db: Data<Pool<Sqlite>>, watchlist: Json<FetchAnimes>) -> HttpResponse {
    let animes = sqlx::query("
    SELECT anime_id
    FROM watch_list_anime
    WHERE watch_list_id = ?;
    ").bind(watchlist.watch_list_id).fetch_all(db.as_ref()).await;

    match animes {
        Ok(row)=>{
            let anime_ids:Result<Vec<i32>> = row.into_iter().map(|s| s.try_get("anime_id")).collect();
            let anime_ids = match anime_ids {
                Ok(vec) => vec,
                Err(e)=>{
                    dbg!(e);
                    return HttpResponse::InternalServerError().into();
                }
            };

            let mut animes:Vec<AnimeResult> = vec![];
            for id  in anime_ids{
                let anime_details = match sqlx::query(
                    "SELECT title_romanji, thumbnail FROM anime WHERE anime_id = ?"
                ).bind(&id).fetch_one(db.as_ref()).await{
                    Ok(id)=>id,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                };

                let title:String = match anime_details.try_get("title_romanji") {
                    Ok(title)=>title,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                }; 

                let picture:String = match anime_details.try_get("thumbnail") {
                    Ok(picture)=>picture,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                };

                animes.push(AnimeResult { id: id, title: title, picture: Some(picture) });

                return HttpResponse::Ok().json(json!(AllAnimeSimple{ anime: animes }))
            }
        }
        Err(e)=>{
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }
    }

    HttpResponse::Ok().into()
}

#[get("check_if_already_in_list")]
pub async fn check_if_an_anime_in_list(db: Data<Pool<Sqlite>>, to_add: Json<AddToList>)->HttpResponse {
    let anime_id = &to_add.anime_id;
    let list_name = &to_add.list_name;
    let user_id = &to_add.user_id;

    let count:i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM watch_anime_list WHERE watch_name = ? AND anime_id = ? AND user_id = ?"
    )
    .bind(list_name)
    .bind(anime_id).bind(user_id)
    .fetch_one(db.as_ref())
    .await.unwrap_or(0);// change this unwarap to actual error handelling

    if count < 1{
        HttpResponse::Ok().json(ExistsInList{
            exists: false
        })
    } else {
        HttpResponse::Ok().json(ExistsInList{
            exists: true
        })
    }
}