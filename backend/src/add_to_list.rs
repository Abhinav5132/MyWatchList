use actix_web::{web::{Data, Json}, HttpResponse};

use crate::*;


// watch list is always 1
// recommend is always 2
// best of all time ranking list is always 3

struct AddToList{
    anime_id: i32, 
    list_id: i32,
}

struct AddListToUser{

    user_id: i32,
    name: String,
    privacy_type: String,
}

#[post("/add-anime-to-list")]
pub async fn add_to_list(db: web::Data<Pool<Sqlite>>,to_add: Json<AddToList>) ->HttpResponse{
    let anime_id = to_add.anime_id;
    let list_id = to_add.list_id;

    match sqlx::query("INSERT INTO watch_list_anime(watch_list_id, anime_id) VALUES (?,?);")
    .bind(list_id)
    .bind(anime_id)
    .execute(db.as_ref()).await {
        Ok(_) => {
            HttpResponse::Ok()
        }

        Err(_) => {
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/remove-form-list")]
pub async fn remove_from_list(db: web::Data<Pool<Sqlite>>,to_add: Json<AddToList>) ->HttpResponse{
    let anime_id = to_add.anime_id;
    let list_id = to_add.list_id;

    match sqlx::query("
    DELETE FROM watch_list_anime 
    WHERE anime_id = ? AND watch_list_id = ?;")
    .bind(anime_id)
    .bind(list_id)
    .execute(db.as_ref()).await {
        Ok(_) => {
            HttpResponse::Ok()
        }

        Err(_) => {
            HttpResponse::InternalServerError()
        }
    }
}

#[post("/add-list-to-user")]
pub async fn create_watch_list(db: Data<Pool<Sqlite>>, to_add: Json<AddListToUser>)-> HttpResponse{
    match sqlx::query("INSERT INTO watch_list(name, user_id, privacy_type) VALUES (?,?,?);")
    .bind(to_add.name)
    .bind(to_add.user_id)
    .bind(to_add.privacy_type)
    .execute(db.as_ref()) {
        Ok(_) => {
            HttpResponse::Ok()
        }

        Err(_) => {
            HttpResponse::InternalServerError()
        }
    }
}


#[post("/remove-list-from-user")]
pub async fn remove_watch_list(db: Data<Pool<Sqlite>>, to_add: Json<AddListToUser>)-> HttpResponse{
    match sqlx::query("DELETE FROM watch_list WHERE name = ? and user_id = ?;")
    .bind(to_add.name)
    .bind(to_add.user_id)
    .execute(db.as_ref()) {
        Ok(_) => {
            HttpResponse::Ok()
        }

        Err(_) => {
            HttpResponse::InternalServerError()
        }
    }
}