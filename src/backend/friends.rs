use actix_web::{HttpRequest, HttpResponse, get, web::{self, Json}};
use sqlx::Pool;
use crate::{backend::add_to_list::AList, try_or};

pub use crate::backend::*;

#[derive(Deserialize)]
pub struct FriendRequest {
    user_id: i64,
    friend_id: i64, 
}

#[derive(Deserialize)]
pub struct RequestId {
    request_id: i64
}

#[derive(Deserialize)]
pub struct FriendId {
    friend_id: i64
}

#[derive(Deserialize, Serialize)]
pub struct Friend {
    friend_id: i64,
    user_name: String,
    user_pfp: String,
}

#[derive(Deserialize, Serialize)]
pub struct AllFriends {
    friends: Vec<Friend>
}

#[derive(Deserialize, Serialize)]
pub struct FullFriend {
    friend_id: i64,
    user_name: String,
    user_pfp: String,
    lists: Vec<AList>
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

#[post("/send_firend_request")]
pub async fn send_firend_request(db: web::Data<Pool<Sqlite>>,request: Json<FriendRequest>, req: HttpRequest) -> HttpResponse {
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), &auth_header).await {
        let count = match sqlx::query("SELECT COUNT(1) FROM friend_requests WHERE sender_id = ? AND receiver_id = ?")
        .bind(request.user_id).bind(request.friend_id).fetch_one(db.as_ref()).await {
            Ok(row) => {
                let cnt: i64 = try_or!(row.try_get(0), HttpResponse::InternalServerError().into());
                cnt
            }
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
            }
        };

        if count == 0 {
            match sqlx::query("INSERT INTO friend_requests(sender_id, receiver_id) VALUES (?,?)")
            .bind(request.user_id).bind(request.friend_id).execute(db.as_ref()).await {
                Ok(_) => { 
                    return HttpResponse::Ok().body("Freind request sent");
                }
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::InternalServerError().into();
                }
            }
        }

        return HttpResponse::Ok().body("Friend request already exists");

    }

    HttpResponse::Unauthorized().into()
}   

#[post("/accept_friend_request")]
pub async fn accept_friend_request(db: web::Data<Pool<Sqlite>>,request: Json<RequestId> ,req: HttpRequest) -> HttpResponse {
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), &auth_header).await {
        match sqlx::query(" 
            WITH req as (SELECT sender_id, receiver_id FROM friend_requests WHERE id = ?)
            INSERT INTO friends (user_1, user_2)
            SELECT 
                CASE WHEN sender_id < receiver_id THEN sender_id ELSE receiver_id END,
                CASE WHEN sender_id < receiver_id THEN receiver_id ELSE sender_id END
            FROM req;

            DELETE FROM friend_requests WHERE id = ?;

        ")
        .bind(request.request_id).fetch_one(db.as_ref()).await {
            Ok(_) => {
                return HttpResponse::Ok().into(); 
            }
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
            }
        };

    }
        HttpResponse::Unauthorized().into()

}

#[post("/decline_friend_request")]
pub async fn decline_friend_request(db: web::Data<Pool<Sqlite>>, request: Json<RequestId> ,req: HttpRequest) -> HttpResponse {

    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), &auth_header).await {
        match sqlx::query(" 
            DELETE FROM friend_requests WHERE id = ?;
        ")
        .bind(request.request_id).fetch_one(db.as_ref()).await {
            Ok(_) => {
                return HttpResponse::Ok().into(); 
            }
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
            }
        };

    }
        HttpResponse::Unauthorized().into()

}

#[post("/remove_friend")]
pub async fn remove_friend(db: web::Data<Pool<Sqlite>>, request: Json<FriendId> ,req: HttpRequest) -> HttpResponse {
    // two ways to do this, return the friendship id and remove based on that or return the other persons id and remove based on that
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap()
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    if verify_token(db.clone(), &auth_header).await {
        match sqlx::query("DELETE FROM friends WHERE id = ?")
        .bind(request.friend_id).execute(db.as_ref()).await {
            Ok(_) => {
                return HttpResponse::Ok().finish();
            }
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

   return HttpResponse::Unauthorized().finish();
}

#[get("/get_all_friends")]
pub async fn get_all_friends(db: web::Data<Pool<Sqlite>>, req: HttpRequest) -> HttpResponse {
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap_or("")
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };
    let user_id = get_userid_from_jwt(auth_header).await;

    if verify_token(db.clone(), &auth_header).await {
        let result = match sqlx::query("
            WITH req as (
                SELECT 
                    CASE WHEN user_1 = ? THEN user_2 ELSE user_1 END AS friend_id
                FROM friends 
                WHERE user_1 = ? OR user_2 = ?
            ) 
            SELECT id, user_name, user_pfp FROM user WHERE id IN (SELECT friend_id FROM req); 
        ").bind(user_id).bind(user_id).bind(user_id).fetch_all(db.as_ref()).await {
            Ok(r) => r,
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
            }
        };

        let mut all_friends= AllFriends{
            friends: vec![]
        };

        for row in result{
            let id: i64 = match row.try_get("id") {
                Ok(i) => i,
                Err(e) => {
                    dbg!(e);
                    continue;
                } 
            };

            let user_name: String = row.try_get("user_name").unwrap_or("UNKNOWN".to_string());
            let user_pfp: String = row.try_get("user_pfp").unwrap_or("UNKNWON".to_string());

            let friend = Friend {
                friend_id: id,
                user_name: user_name,
                user_pfp: user_pfp
            };

            all_friends.friends.push(friend);
        }
        return HttpResponse::Ok().json(all_friends).into();
    }
    return HttpResponse::Unauthorized().into();
}

#[post("/get_friend_profile")]
pub async fn view_friend_profile(db: web::Data<Pool<Sqlite>>, req: HttpRequest, request: Json<FriendId> ) -> HttpResponse { // friend table id
    let auth_header =  match req.headers().get("Authorization") {
        Some(token) => {
            token.to_str().unwrap_or("")
        }
        None=>{
            return HttpResponse::Unauthorized().into();
        }
    };

    let user_id = get_userid_from_jwt(auth_header).await;

    // get the user id using their friend id and then use the user id to get user_details and then get their publicly available list 

    if verify_token(db.clone(), &auth_header).await {
        let result = match sqlx::query("
            SELECT 
                CASE WHEN user_1 = ? THEN user_2 ELSE user_1 END AS friend_id
            FROM friends 
            WHERE  id = ? AND (user_1 = ? OR user_2 = ?)").bind(user_id).bind(request.friend_id)
            .bind(user_id).bind(user_id).fetch_one(db.as_ref()).await
            {
            Ok(r) => r,
            Err(e) => {
                dbg!(e);
                return HttpResponse::InternalServerError().into();
            }
        };

        if result.is_empty() {
            return HttpResponse::Forbidden().body("Not Friends");
        }
        let friend_id = result.try_get("friend_id").unwrap_or(-164);
        if friend_id != -1 {
            let row = match sqlx::query("SELECT id, user_name, user_pfp FROM user WHERE id = ?")
            .bind(friend_id).fetch_one(db.as_ref()).await {
                Ok(r) => r,
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::InternalServerError().into();
                }
            };

            let id: i64 = match row.try_get("id") {
                Ok(i) => i,
                Err(e) => {
                    dbg!(e);
                    friend_id
                } 
            };

            let user_name: String = row.try_get("user_name").unwrap_or("UNKNOWN".to_string());
            let user_pfp: String = row.try_get("user_pfp").unwrap_or("UNKNWON".to_string());

            let row = match sqlx::query("SELECT id, name, description, list_image FROM watch_list WHERE user_id = ? 
                AND (privacy_type = ? OR privacy_type = ?)")
                .bind(friend_id).bind(WatchListType::FriendsOnly.string())
                .bind(WatchListType::Public.string()).fetch_all(db.as_ref()).await {
                Ok(r) => {
                    r
                }
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::InternalServerError().into();
                }
            };
            let mut lists = vec![];
            for r in row{
                let id = match r.try_get("id") {
                    Ok(i) => i,
                    Err(e) => {
                        dbg!(e);
                        continue;
                    }
                };

                let name = r.try_get("name").unwrap_or("UNKNWON".to_string());
                let description = r.try_get("description").unwrap_or("UNKNOWN".to_string());
                let list_image = r.try_get("list_image").unwrap_or("UNKNOWN".to_string());

                let alist = AList{
                    id: id,
                    name: name,
                    image: list_image,
                    description: description
                };
                lists.push(alist);
            }

            let fullfriend = FullFriend{
                friend_id: id,
                user_name: user_name,
                user_pfp: user_pfp,
                lists: lists
            };

            return HttpResponse::Ok().json(fullfriend).into();

        }
        else {
            return HttpResponse::InternalServerError().into();
        }
    }

    return HttpResponse::Unauthorized().into();
}

