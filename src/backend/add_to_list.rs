use std::{fs, io::Cursor};
use actix_web::{web::{Data, Json, Query}, HttpRequest, HttpResponse};
use base64::{engine::general_purpose, Engine};
use openssl::pkey::Public;
use rand_core::impls;
use reqwest::Client;
use serde_json::json;
use sqlx::sqlite::SqliteRow;
use image::ImageReader;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageError, RgbaImage};
use crate::{backend::*, try_or};


// watch list is always 1
// recommend is always 2
// best of all time ranking list is always 3
//IMAGE FORMAT MUST ALWAYS BE A PNG FOR LATER WHEN I IMPLEMENT USER ADDED IMAGES 

#[derive(Deserialize)]
pub struct AddToList{
    list_id: i64,
    anime_id: i64, 
    list_name: String,
    user_id: i64,
    rank: Option<i64>
}

pub enum WatchListType {
    Public,
    Private,
    FriendsOnly
}

impl WatchListType {
    pub fn string(&self)->String{
        match self {
            WatchListType::Public => "Public".to_string(),
            WatchListType::Private => "Private".to_string(),
            WatchListType::FriendsOnly => "FriendsOnly".to_string()
        }
    }
}

#[derive(Serialize)]
struct AList{
    name: String,
    id: i64,
    image: String,
    description: String
}

#[derive(Serialize)]
struct ACompleteList{
    name: String,
    image: String,
    is_ranked: i32,
    is_user_image: i32,
    privacy_type: String,
    description: String
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
    page_no: i32,
    per_page: i32
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
    is_ranked: i32,
    image: Vec<u8>,
    is_user_image: i32,
    description: String,
}

#[derive(Deserialize)]
pub struct EditListPerUser{

    user_id: i64,
    list_id: i64,
    new_name: String,
    new_privacy_type: String,
    new_is_ranked: i32,
    new_image: String,
    is_user_image: i32,
    description: String
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

#[derive(Deserialize)]
pub struct ListId{
    list_id: i64,
}

pub fn file_to_blob_with_path(path: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    Ok(bytes)
}

pub async fn file_to_blob_with_link(path: &str) -> Result<Vec<u8>, reqwest::Error> {
    let client = Client::new();
    let resp = client.get(path).send().await?; 
    if resp.status().is_success() {
       Ok(resp.bytes().await?.to_vec())
    }else {
        Err(resp.error_for_status().unwrap_err())
    }
}


pub async fn re_order_list_on_addition(db: Data<Pool<Sqlite>>, rank: i64, list_name: String, user_id:i64) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("
    UPDATE watch_list_anime
    SET rank = rank + 1000
    WHERE user_id = ? AND watch_name = ? AND rank >= ?;
    ")
        .bind(user_id)
        .bind(&list_name)
        .bind(rank)
        .execute(&mut *tx)
        .await?;

    sqlx::query("
            UPDATE watch_list_anime
            SET rank = rank - 999
            WHERE user_id = ? AND watch_name = ? AND rank >= (? + 1000);
        ")
        .bind(user_id)
        .bind(&list_name)
        .bind(rank)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())

}

pub async fn re_order_list_on_remove(db: Data<Pool<Sqlite>>, rank: i64, list_name: String, user_id:i64) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("
    UPDATE watch_list_anime 
    SET rank = rank - 1 
    WHERE user_id = ? AND watch_name = ? AND rank > ?") .bind(user_id)
        .bind(&list_name)
        .bind(rank)
        .execute(&mut *tx)
        .await?; 
    tx.commit().await?;
    Ok(())
}

pub fn combine_images_in_a_grid(blobs: Vec<Vec<u8>>) -> Result<Vec<u8>, ImageError>{

    // convert blobs to images
    let mut images: Vec<DynamicImage> = Vec::new();
    for b in blobs {
        let img = ImageReader::new(Cursor::new(b))
            .with_guessed_format()?   // detect PNG, JPG, etc.
            .decode()?;               // into DynamicImage
        images.push(img);
    }

    let (w, h) = images[0].dimensions();
    for img in &mut images {
        *img = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    }

    let (rows, cols) = match images.len() {
        1 => (1, 1),
        2 => (1, 2), // side by side
        3 | 4 => (2, 2), // 2x2 grid
        n => {
            // For >4, make it roughly square
            let cols = (n as f64).sqrt().ceil() as u32;
            let rows = ((n as f64) / cols as f64).ceil() as u32;
            (rows, cols)
        }
    };
    // grate a blank canvas
    let mut grid:RgbaImage = ImageBuffer::new(w * cols, h * rows);

    for (idx, img) in images.iter().enumerate() {
        let row = (idx as u32) / cols;
        let col = (idx as u32) % cols;
        grid.copy_from(img, col * w, row * h)?;
    }
    

    // save the new image buffer to an in-memeory buffer
    let mut buffer = Cursor::new(Vec::new());
    grid.write_to(&mut buffer, image::ImageFormat::Png)?;

    Ok(buffer.into_inner())
} 

pub async fn genereate_grid(db: Data<Pool<Sqlite>>, list_name: &String, user_id:i64, is_ranked: bool) -> Result<Vec<u8>>{ // change this to vec u8{
    dbg!("Generate grid ran");
    let count:u64 = match sqlx::query_scalar("SELECT COUNT(*) as cnt FROM watch_list_anime WHERE watch_name = ? AND user_id = ?").bind(list_name) 
    .bind(user_id).fetch_one(db.as_ref()).await {
        Ok(c) => c,
        Err(e) => {
            dbg!(e);
            0
        }
    };
    dbg!(count);
    let mut image_bytes:Result<Vec<u8>> = Ok(vec![]); 
    match count {
        0 => {
            image_bytes = file_to_blob_with_path("assets/images.png");
        },
        1 => {
            let watch_name = list_name.clone();
            let top_images = sqlx::query("SELECT a.mediumImage 
            FROM watch_list_anime wla 
            JOIN anime a ON a.id = wla.anime_id
            WHERE wla.user_id = ? 
            AND wla.watch_name = ?
            LIMIT 1;   
            ").bind(user_id).bind(watch_name).fetch_all(db.as_ref()).await?;
            for top_image in top_images {
                let image:String = top_image.try_get("mediumImage")?;
                let bytes = match file_to_blob_with_link(&image).await {
                    Ok(img) => Some(img),
                    Err(e) => {
                        dbg!(e);
                        None
                    }
                };

                image_bytes = match bytes {
                    Some(b) => Ok(b),
                    None => Err(sqlx::Error::RowNotFound),
                };
            }

        },
        2..=3 => {
            dbg!("Trying to genereate 2x2 grid");
            let vec_image = sqlx::query("SELECT a.thumbnail 
            FROM watch_list_anime wla 
            JOIN anime a ON a.id = wla.anime_id
            WHERE wla.user_id = ? 
            AND wla.watch_name = ?
            LIMIT 2;   
            ").bind(user_id).bind(list_name).fetch_all(db.as_ref()).await?;
            let mut images:Vec<Vec<u8>> = vec![];
            for top_image in vec_image {
                let image:String = top_image.try_get("thumbnail")?;
                match file_to_blob_with_link(&image).await {
                    Ok(img) => images.push(img),
                    Err(e) => {
                        dbg!(e);
                    }
                }; 
            }

            image_bytes = match combine_images_in_a_grid(images) {
                Ok(new_image) => Ok(new_image),
                Err(e) => {
                    dbg!(e);
                    dbg!("Failed to convert the images into a grid");
                    Err(sqlx::Error::RowNotFound)
                }
            }

        }
        4.. => {
            dbg!("Trying to generate 44 grid");
            let sort_by = if is_ranked { "rank" } else { "rank" }.to_string(); //IMPORTANT CHANGE TO DATE_ADDED ONCE IMPLEMENTED IN THE DATABASE

            let vec_image = sqlx::query("SELECT a.thumbnail 
                FROM watch_list_anime wla 
                JOIN anime a ON a.id = wla.anime_id
                WHERE wla.user_id = ? 
                AND wla.watch_name = ?
                ORDER BY ?
                LIMIT 4;   
                ").bind(user_id).bind(list_name).bind(sort_by).fetch_all(db.as_ref()).await?;
            let mut images:Vec<Vec<u8>> = vec![];
            for top_image in vec_image {
                let image:String = top_image.try_get("thumbnail")?;
                match file_to_blob_with_link(&image).await {
                    Ok(img) => images.push(img),
                    Err(e) => {
                        dbg!(e);
                    }
                }; 
            }

            image_bytes = match combine_images_in_a_grid(images) {
                Ok(new_image) => Ok(new_image),
                Err(e) => {
                    dbg!(e);
                    dbg!("Failed to convert the images into a grid");
                    Err(sqlx::Error::RowNotFound)
                }
            }

        }

    }
    image_bytes
}

#[get("/get-if-ranked")]
pub async fn get_if_ranked(db: Data<Pool<Sqlite>>, details: Json<IfRanked>, req: HttpRequest ) -> HttpResponse {
    let auth_header = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), auth_header).await {
        if get_userid_from_jwt(auth_header).await != details.user_id {
            return HttpResponse::Unauthorized().into();
        } else {
        let result = sqlx::query("SELECT is_ranked FROM watch_list WHERE name = ? and user_id = ?;" )
        .bind(&details.list_name)
        .bind(details.user_id)
        .fetch_one(db.as_ref()).await;
        let is_ranked: i32 = match result {
            Ok(row) => row.try_get("is_ranked").unwrap_or(0),
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
                }
            };
            if is_ranked == 1 {
                match sqlx::query("SELECT rank FROM watch_list_anime 
                            WHERE user_id = ? AND watch_name = ? 
                            ORDER BY rank DESC 
                            LIMIT 1;")
                            .bind(details.user_id).bind(details.list_name.clone())
                            .fetch_optional(db.as_ref()).await{
                Ok(Some(rank)) => {
                    let last_rank = rank.try_get("rank").unwrap_or(1);
                    HttpResponse::Ok().json(IsRanked{
                    is_ranked: is_ranked,
                    last_rank: last_rank
                    }).into()
                }

                Ok(None) => {
                    HttpResponse::Ok().json(IsRanked {
                        is_ranked: is_ranked,
                        last_rank: 0
                    })
                }

                Err(r)=> {
                    dbg!(r);
                    return HttpResponse::InternalServerError().into();
                }
            }
            }else {
                HttpResponse::Ok().json(IsRanked{
                    is_ranked: is_ranked,
                    last_rank: 0
                })
            }

            }
        }else {
            return HttpResponse::Unauthorized().into();
        }
                    
}

#[post("/add-anime-to-list")] // must verify the users identity before it adds 
pub async fn add_anime_to_list(db: web::Data<Pool<Sqlite>>,to_add: Json<AddToList>, req: HttpRequest) ->HttpResponse{
    
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), &auth_header).await && get_userid_from_jwt(&auth_header).await == to_add.user_id {
        let list_id = &to_add.anime_id;
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
            "SELECT COUNT(1) FROM watch_list_anime WHERE list_id = ? AND anime_id = ? AND user_id = ?"
        )
        .bind(list_id)
        .bind(anime_id).bind(user_id)
        .fetch_one(db.as_ref())
        .await {
            Ok(c) => c,
            
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
            }

        };

        let reorder_result =re_order_list_on_addition(db.clone(), to_add.rank.unwrap_or(1), to_add.list_name.clone(), to_add.user_id).await;
        if let Ok(_) = reorder_result{
                //dbg!(&count);
            if count == 0 {
                match sqlx::query("INSERT INTO watch_list_anime(watch_name, anime_id, user_id, rank, list_id) VALUES (?,?,?,?,?);")
                .bind(list_name)
                .bind(anime_id)
                .bind(user_id)
                .bind(rank)
                .bind(list_id)
                .execute(db.as_ref()).await {
                Ok(_) => {
                    dbg!("Excecuted properly");

                    let image = genereate_grid(db.clone(), list_name, *user_id, true).await;

                    match image {
                        Ok(img) =>{
                            dbg!("Image inserted into the list");
                        let _ = sqlx::query("UPDATE watch_list SET list_image = ?, is_user_image = ? WHERE name = ?;").bind(img).bind(false).bind(list_name).execute(db.as_ref()).await;
                        }
                        Err(e) => {
                            dbg!(e);
                        }
                    }
                        return HttpResponse::Ok().into();
                    }

                Err(e) => {
                    dbg!(e);
                    return HttpResponse::InternalServerError().into();
                    }
                }
            } else {
                dbg!("Anime is alreadt in the list");
                return HttpResponse::Conflict().body("Anime is already in list");
            }
        } else {
            dbg!("result is false from reorder list");
            return HttpResponse::InternalServerError().into();
        }
        } else {
            return HttpResponse::Unauthorized().into();
        }

}

#[post("/remove-form-list")]
pub async fn remove_from_list(db: web::Data<Pool<Sqlite>>,to_add: Json<AddToList>, req: HttpRequest) ->HttpResponse{
    
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if !verify_token(db.clone(),&auth_header).await || get_userid_from_jwt(&auth_header).await != to_add.user_id {
        return HttpResponse::Unauthorized().into();
    }
    match sqlx::query("DELETE FROM watch_list_anime WHERE anime_id = ?, user_id = ?, list_id = ?") 
        .bind(to_add.anime_id).bind(to_add.user_id).bind(&to_add.list_id).execute(db.as_ref()).await
    {
        Ok(result) => {
            if result.rows_affected() <= 0 {
                return HttpResponse::InternalServerError().into();
                }
            match re_order_list_on_remove(db.clone(), to_add.rank.unwrap_or(1), to_add.list_name.clone(), to_add.user_id).await {
                Ok(_) => {
                    return HttpResponse::Ok().into();
                }   
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::Ok().into();
                }
            }
            }
        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }
    }
}

#[post("/add-list-to-user")] // needs verification
pub async fn create_watch_list(db: Data<Pool<Sqlite>>, to_add: Json<AddListToUser>)-> HttpResponse{
    match create_list(&db, &to_add.name, &to_add.user_id, &to_add.privacy_type, to_add.is_ranked, &to_add.image, to_add.is_user_image, &to_add.description).await { // this was a shitty way to write this
        Ok(_) => {
            HttpResponse::Ok().into()
        }

        Err(_) => {
            HttpResponse::InternalServerError().into()
        }
    }
}

pub async fn create_list(db: &Pool<Sqlite>, name: &String, user_id:&i64, privacy_type: &String, is_ranked: i32, image: &Vec<u8>, is_user_image: i32, description: &String)->Result<(), sqlx::Error>{

    sqlx::query("INSERT INTO watch_list(name, description, user_id, privacy_type, is_ranked, list_image, is_user_image) VALUES (?,?,?,?,?,?,?);")
    .bind(name)
    .bind(description)
    .bind(user_id)
    .bind(privacy_type)
    .bind(is_ranked)
    .bind(image)
    .bind(is_user_image)
    .execute(db).await?;

    Ok(())

}

#[post("/remove-list-from-user")]
pub async fn remove_watch_list(db: Data<Pool<Sqlite>>, to_add: Json<AddListToUser>) -> HttpResponse{
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

#[post("/edit-watch-list-from-user")]
pub async fn edit_watch_list(db: Data<Pool<Sqlite>>, to_add: Json<EditListPerUser>, req: HttpRequest) -> HttpResponse {

    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), &auth_header).await && get_userid_from_jwt(&auth_header).await == to_add.user_id {

        let mut tx = match db.begin().await {
            Ok(transaction) => transaction,
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
            }
        };
        match sqlx::query("UPDATE watch_list 
            SET name = ? AND 
            description = ? AND
            privacy_type = ? AND
            is_ranked = ? AND list_image = ? AND 
            is_user_image = ? WHERE user_id = ? AND id = ? ")
            .bind(&to_add.new_name).bind(&to_add.description).bind(&to_add.new_privacy_type).bind(to_add.new_is_ranked).bind(&to_add.new_image)
            .bind(to_add.is_user_image).bind(to_add.user_id).bind(&to_add.list_id).execute(&mut *tx).await {
                Ok(_) => {
                    match sqlx::query("UPDATE OR IGNORE watch_list_anime SET watch_name = ? WHERE list_id = ? AND user_id = ?")
                    .bind(&to_add.new_name).bind(&to_add.list_id).bind(to_add.user_id).execute(&mut *tx).await
                     {
                        Ok(_) => {
                            let _ = tx.commit().await;
                           return HttpResponse::Ok().into();
                        }

                        Err(e) => {
                            dbg!(e);
                            let _ = tx.rollback().await;
                            return HttpResponse::InternalServerError().into();

                        }
                    }
                } ,
                Err(e) => {
                    dbg!(e);
                    let _ = tx.rollback().await;
                    return HttpResponse::InternalServerError().into();

                } 
            };

    } else {
       return HttpResponse::Unauthorized().into();
    }
}

#[get("/fetch-all-lists")] //TODO add a function to get all lists that you have the privilages to view for another person
pub async fn fetch_all_lists(db: Data<Pool<Sqlite>>, user: Json<FetchLists>, req: HttpRequest) -> HttpResponse {
    let per_page = user.per_page;
    let offset = (user.page_no - 1) * per_page;
    let auth_header = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), auth_header).await {
        if get_userid_from_jwt(auth_header).await != user.user_id{
            return HttpResponse::Unauthorized().into();
        }

        let lists = match sqlx::query("Select name, id, privacy_type, list_image, description
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

            let image:Vec<u8> = match r.try_get("list_image") {
                Ok(img) => img,
                Err(e) => {
                    dbg!(e);
                    continue;
                }
            };
            let description = match r.try_get("description") {
                Ok(des) => des,
                Err(e) => {
                    dbg!(e);
                    continue;
                }
            };

            let base64_img = general_purpose::STANDARD.encode(&image);
            let data_url = format!("data:image/png;base64,{}", base64_img);

            let alist = AList{
                name: name,
                id: id,
                image: data_url,
                description: description
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
                    "SELECT title_romanji, LargeImage FROM anime WHERE id = ?"
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

                let picture:String = match anime_details.try_get("LargeImage") {
                    Ok(picture)=>picture,
                    Err(e)=>{
                        dbg!(e);
                        return HttpResponse::InternalServerError().into();
                    }
                };

                animes.push(AnimeResult { id: id, title: title, largeImage: Some(picture) });

                
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


#[get("/get_list_details")]
pub async fn get_list_details(db: Data<Pool<Sqlite>>, req: HttpRequest, query: Query<ListId> ) -> HttpResponse {
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    let user_id = get_userid_from_jwt(auth_header).await;

    let row = try_or!(
        sqlx::query("SELECT name, is_ranked, is_user_image, list_image, privacy_type, description
         FROM watch_list WHERE user_id = ? AND id = ?;")
        .bind(user_id)
        .bind(&query.list_id)
        .fetch_one(db.as_ref()).await, HttpResponse::InternalServerError().finish()
    );

    let name = try_or!(row.try_get("name"), HttpResponse::InternalServerError().finish());
    let is_ranked:i32 = try_or!(row.try_get("is_ranked"), HttpResponse::InternalServerError().finish());
    let is_user_image:i32 = try_or!(row.try_get("is_user_image"), HttpResponse::InternalServerError().finish());
    let list_image:Vec<u8> = try_or!(row.try_get("list_image"), HttpResponse::InternalServerError().finish());
    let privacy_type:String = try_or!(row.try_get("privacy_type"), HttpResponse::InternalServerError().finish());
    let description: String = try_or!(row.try_get("description"), HttpResponse::InternalServerError().finish());

    let base64_img = general_purpose::STANDARD.encode(&list_image);
    let data_url = format!("data:image/png;base64,{}", base64_img);

    HttpResponse::Ok().json(&ACompleteList{
        name: name,
        is_ranked:is_ranked,
        is_user_image: is_user_image,
        image: data_url,
        privacy_type: privacy_type,
        description: description
    })
}