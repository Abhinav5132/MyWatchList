use actix_web::{web::{BufMut, Data, Json, Query}, HttpResponse, HttpRequest};
use serde_json::json;
use sqlx::sqlite::SqliteRow;

use crate::*;


// watch list is always 1
// recommend is always 2
// best of all time ranking list is always 3

#[derive(Deserialize)]
pub struct AddToList{
    anime_id: i64, 
    list_name: String,
    user_id: i64,
    rank: Option<i64>
}

#[derive(Serialize)]
struct AList{
    name: String,
    id: i64
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
    user_id: i64, 
    page_no: i32
}

#[derive(Deserialize)]
struct FetchAnimes{
    watch_list_name: String,
    user_id: i64,
    page_no: i64,
}

#[derive(Deserialize)]
pub struct AddListToUser{

    user_id: i64,
    name: String,
    privacy_type: String,
    is_ranked: i32
}

#[derive(Serialize)]
pub struct ExistsInList{
    exists: bool
}

#[derive(Deserialize)]
pub struct IfRanked{
    list_name: String,
    user_id: i64,
}


#[derive(Serialize)]
pub struct IsRanked{
    is_ranked: i32,
    last_rank: i32
}

pub async fn re_order_list(db: Data<Pool<Sqlite>>, rank: i64, list_name: String, user_id:i64) -> Result<(), ()> {
    let result = sqlx::query("
    UPDATE watch_list_anime 
    SET rank = rank + 1
    WHERE user_id = ? AND
    watch_name = ? AND
    rank >= ?;
    ").bind(user_id).bind(list_name).bind(rank).execute(db.as_ref()).await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            dbg!(e);
            Err(())
        }
    }

}

#[get("/get-if-ranked")]
pub async fn get_if_ranked(db: Data<Pool<Sqlite>>, details: Json<IfRanked>) -> HttpResponse {
    match sqlx::query("SELECT is_ranked FROM watch_list WHERE name = ? and user_id = ?;" )
        .bind(&details.list_name)
        .bind(details.user_id)
        .fetch_one(db.as_ref()).await
        {
            Ok(cou) => {
                let count = cou.try_get("is_ranked").unwrap_or(0); // for now assuming not ranking cuz it should technicall never reach this as is_ranked cannot be null

                    match sqlx::query("SELECT rank FROM watch_list_anime 
                                    WHERE user_id = ? AND watch_name = ? 
                                    ORDER BY rank DESC 
                                    LIMIT 1;")
                                    .bind(details.user_id).bind(details.list_name.clone())
                                    .fetch_one(db.as_ref()).await{
                        Ok(rank) => {
                            let last_rank = rank.try_get("rank").unwrap_or(1);
                            HttpResponse::Ok().json(IsRanked{
                            is_ranked: count,
                            last_rank: last_rank
                            }).into()
                        }

                        Err(r)=> {
                            dbg!(r);
                            return HttpResponse::InternalServerError().into();
                        }
                    }
                    
                }

            Err(e) => {
                dbg!(e); // add actiall error handeling here
                return HttpResponse::InternalServerError().into();
            }
            
        }
}

#[post("/add-anime-to-list")] // must verify the users identity before it adds 
pub async fn add_anime_to_list(db: web::Data<Pool<Sqlite>>,to_add: Json<AddToList>) ->HttpResponse{
    let anime_id = &to_add.anime_id;
    let list_name = &to_add.list_name;
    let user_id = &to_add.user_id;
    let rank = match to_add.rank {
        Some(rank) => rank,
        None => -1
    };
    dbg!(&anime_id);
    dbg!(&list_name);
    dbg!(&user_id);
    let count:i64 = match sqlx::query_scalar(
        "SELECT COUNT(1) FROM watch_list_anime WHERE watch_name = ? AND anime_id = ? AND user_id = ?"
    )
    .bind(list_name)
    .bind(anime_id).bind(user_id)
    .fetch_one(db.as_ref())
    .await {
        Ok(c) => c,
        
        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }

    };

    let reorder_result =re_order_list(db.clone(), to_add.rank.unwrap_or(1), to_add.list_name.clone(), to_add.user_id).await;
    if let Ok(result ) = reorder_result{
            //dbg!(&count);
        if count == 0 {
            match sqlx::query("INSERT INTO watch_list_anime(watch_name, anime_id, user_id, rank) VALUES (?,?,?,?);")
            .bind(list_name)
            .bind(anime_id)
            .bind(user_id)
            .bind(rank)
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
            dbg!("Anime is alreadt in the list");
            HttpResponse::Conflict().body("Anime is already in list")
        }
    }else {
        dbg!("result is false from reorder list");
        HttpResponse::InternalServerError().into()
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
    match create_list(&db, &to_add.name, &to_add.user_id, &to_add.privacy_type, to_add.is_ranked).await {
        Ok(_) => {
            HttpResponse::Ok().into()
        }

        Err(_) => {
            HttpResponse::InternalServerError().into()
        }
    }
}

pub async fn create_list(db: &Pool<Sqlite>, name: &String, user_id:&i64, privacy_type: &String, is_ranked: i32)->Result<(), sqlx::Error>{
    sqlx::query("INSERT INTO watch_list(name, user_id, privacy_type, is_ranked) VALUES (?,?,?,?);")
    .bind(name)
    .bind(user_id)
    .bind(privacy_type)
    .bind(is_ranked)
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
// this also needs to incorporate pages 
#[get("/fetch-all-lists")]
pub async fn fetch_all_lists(db: Data<Pool<Sqlite>>, user: Json<FetchLists>, req: HttpRequest) -> HttpResponse {
    let per_page = 10;
    let offset = (user.page_no - 1) * per_page;
    let auth_header = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(auth_header).await {
        let mut lists:Vec<SqliteRow> = vec![];
        if get_userid_from_jwt(auth_header).await != user.user_id{
            lists = match sqlx::query("
            Select name, id, privacy_type 
            FROM watch_list 
            WHERE user_id = ? AND privacy_type = ? 
            LIMIT ? OFFSET ?;")
            .bind(&user.user_id)
            .bind("Public")
            .bind(per_page)
            .bind(offset)
            .fetch_all(db.as_ref()).await{
                Ok(row) =>{
                    row
                },
                Err(e)=>{
                    dbg!(e);
                    return HttpResponse::InternalServerError().into();
                }

            };
        }else{
            lists = match sqlx::query("Select name, id, privacy_type 
            FROM watch_list 
            WHERE user_id = ?
            LIMIT ? OFFSET ?;")
            .bind(&user.user_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(db.as_ref()).await{
                Ok(row) =>{
                    row
                },
                Err(e)=>{
                    dbg!(e);
                    return HttpResponse::InternalServerError().into();
                }

            };
        }

        let mut all_list: AllListSimple = AllListSimple{list: vec![]};
        for r in lists{
            let name:String =match r.try_get("name") {
                Ok(n) => n,
                Err(e) => {
                    dbg!(e);
                    continue}
            }; 
            let id:i64 = match r.try_get("id") {
                Ok(i) => i,
                Err(e)=>
                {
                    dbg!(e);
                    continue;
                }
            };

            let alist = AList{
                name: name,
                id: id
            };
            all_list.list.push(alist);
        }
        
        HttpResponse::Ok().json(json!(all_list))
    } else {
        HttpResponse::Unauthorized().into()
    }

} 


// used to display all anime in a list_page
//maybe change this to include description
#[get("/get-animes-from-list")]
pub async fn fetch_all_anime_from_list(db: Data<Pool<Sqlite>>, watchlist: Json<FetchAnimes>) -> HttpResponse {
    let per_page = 10;
    let offset = (watchlist.page_no - 1) * per_page;
    let animes = sqlx::query("
    SELECT anime_id
    FROM watch_list_anime
    WHERE watch_name = ? and user_id = ?
    LIMIT ? OFFSET ?;
    ").bind(&watchlist.watch_list_name)
    .bind(&watchlist.user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(db.as_ref()).await;

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
                    "SELECT title_romanji, picture FROM anime WHERE id = ?"
                ).bind(&id).fetch_one(db.as_ref()).await{
                    Ok(id)=>id,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into(); //TODO later change this to only fail for the next one and send the errror upstream for handeling 
                    }
                };

                let title:String = match anime_details.try_get("title_romanji") {
                    Ok(title)=>title,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                }; 

                let picture:String = match anime_details.try_get("picture") {
                    Ok(picture)=>picture,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                };

                animes.push(AnimeResult { id: id, title: title, picture: Some(picture) });

                
            }
           return HttpResponse::Ok().json(json!(AllAnimeSimple{ anime: animes })) 
        }
        Err(e)=>{
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }
    }
    
}

#[get("/check_if_already_in_list")]
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