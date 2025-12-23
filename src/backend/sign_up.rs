use crate::backend::add_to_list::{
    WatchListType, create_list, encode_to_base64, file_to_blob_with_path,
};
pub use crate::backend::*;
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
pub use authenticate::pwd_to_hash;
pub use serde_json::json;
pub use std::fs;
//implement profile pic later
#[derive(Deserialize)]
pub struct SignUpStruct {
    user_name: String,
    user_password: String,
    user_email: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Deserialize)]
pub struct CheckUserNameAvailability {
    username: String,
}

#[derive(Serialize)]
pub struct CheckUserNameAvailabilityResponse {
    available: bool,
}

#[derive(Deserialize)]
pub struct CheckEmailAvailability {
    email: String,
}

#[derive(Serialize)]
pub struct CheckEmailAvailabilityResponse {
    available: bool,
}
#[post("/Signup")]
pub async fn sign_up_fn(
    db: web::Data<Pool<Sqlite>>,
    credentials: web::Json<SignUpStruct>,
) -> HttpResponse {
    let entered_pwd = &credentials.user_password;
    let hashed_pwd = match pwd_to_hash(entered_pwd) {
        Ok(pwd) => pwd,
        Err(_) => {
            dbg!("Unable to login 1");
            return HttpResponse::InternalServerError().json(json!({
                "status": "Internal error converting pwd to hash"
            }));
        }
    };
    dbg!(&credentials.user_name);
    dbg!(&entered_pwd);
    let no_pfp_blob = match file_to_blob_with_path("assets/No_pfp.jpg") {
        Ok(pfp) => pfp,
        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }
    };

    let no_pfp = match encode_to_base64(no_pfp_blob).await {
        Some(pfp) => pfp,
        None => {
            dbg!("fucking failed for some reason");
            return HttpResponse::InternalServerError().into();
        }
    };

    let user_id = match sqlx::query(
        "INSERT INTO user (user_name, user_email, user_password, user_pfp) VALUES (?,?,?,?);",
    )
    .bind(&credentials.user_name)
    .bind(&credentials.user_email)
    .bind(hashed_pwd)
    .bind(no_pfp)
    .execute(db.as_ref())
    .await
    {
        Ok(rows) => rows.last_insert_rowid(),
        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "Internal error"
            }));
        }
    };

    let access_token = match generate_access_token(user_id).await {
        Ok(tok) => tok,
        Err(_) => {
            dbg!("Unable to login 3");
            return HttpResponse::InternalServerError().json(json!({
                "status": "Internal error generating token"
            }));
        }
    };

    let refresh_token = match generate_refresh_token(user_id).await {
        Ok(tok) => tok,
        Err(e) => {
            dbg!("Unable to login", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "Internal error generating token"
            }));
        }
    };

    let query = sqlx::query(
        "
    UPDATE user SET (user_access_token, user_refresh_token) = (?, ?) WHERE id = ?;
    ",
    )
    .bind(&access_token)
    .bind(&refresh_token)
    .bind(user_id)
    .execute(db.as_ref())
    .await;

    let default_image = match file_to_blob_with_path("assets/images.png") {
        Ok(image) => {
            let a = match encode_to_base64(image).await {
                Some(image) => image,
                None => {
                    dbg!("Something is coocked");
                    "".to_string()
                }
            };
            dbg!(&a);
            a
        }
        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }
    };

    // adding the basic lists to the user after account creation
    match create_list(
        db.as_ref(),
        &"Watch_List".to_string(),
        &user_id,
        &WatchListType::Private.string(),
        0,
        &default_image,
        0,
        &"".to_string(),
    )
    .await
    {
        // make these have actuall distinct images later.
        Ok(_) => (),
        Err(e) => {
            dbg!(e);
            //actual error handeling here later
        }
    }

    match create_list(
        db.as_ref(),
        &"Recommended".to_string(),
        &user_id,
        &WatchListType::Public.string(),
        1,
        &default_image,
        0,
        &"".to_string(),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            dbg!(e);
            //actual error handeling here later
        }
    }

    match create_list(
        db.as_ref(),
        &"Private_list".to_string(),
        &user_id,
        &WatchListType::FriendsOnly.string(),
        1,
        &default_image,
        0,
        &"".to_string(),
    )
    .await
    {
        // only for debugging remove later
        Ok(_) => (),
        Err(e) => {
            dbg!(e);
            //actual error handeling here later
        }
    }

    match query {
        Ok(_) => {
            dbg!("login successful 4");
            HttpResponse::Ok().json(AuthResponse {
                access_token: access_token,
                refresh_token: refresh_token,
                expires_in: (chrono::Utc::now() + chrono::Duration::minutes(3)).timestamp() as u64,
            })
        }

        Err(_) => {
            dbg!("Unable to login");
            return HttpResponse::InternalServerError().json(serde_json::json!({
            "Status": "Unable to login"
            }));
        }
    }
}

#[get("/check_username_availability")]
pub async fn check_username_availability(
    db: Data<Pool<Sqlite>>,
    username: Json<CheckUserNameAvailability>,
) -> HttpResponse {
    let count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM user WHERE user_name = ?;")
        .bind(&username.username)
        .fetch_one(db.as_ref())
        .await
    {
        Ok(c) => c,

        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }
    };

    if count != 0 {
        return HttpResponse::Ok().json(&CheckUserNameAvailabilityResponse { available: false });
    }
    return HttpResponse::Ok().json(&CheckUserNameAvailabilityResponse { available: true });
}

#[get("/check_email_availability")]
pub async fn check_email_availability(
    db: Data<Pool<Sqlite>>,
    email: Json<CheckEmailAvailability>,
) -> HttpResponse {
    let count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM user WHERE user_email = ?")
        .bind(&email.email)
        .fetch_one(db.as_ref())
        .await
    {
        Ok(c) => c,
        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().into();
        }
    };

    if count != 0 {
        return HttpResponse::Ok().json(&CheckEmailAvailabilityResponse { available: false });
    } else {
        return HttpResponse::Ok().json(&CheckEmailAvailabilityResponse { available: true });
    }
}
