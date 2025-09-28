use actix_web::{web::Json, HttpRequest, HttpResponse};
use base64::{engine::general_purpose, Engine};

pub use crate::backend::*;
use crate::try_or;

#[derive(Serialize, Deserialize)]
pub struct UserDetails{
    username: String,
    user_email: String,
    user_pfp: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserId{
    user_id: i64,
}



#[post("/get_user_details")]
pub async fn get_user_details(db: web::Data<Pool<Sqlite>>, user_id: Json<UserId>) -> HttpResponse{
    let row = try_or!(sqlx::query("SELECT user_name, user_email, user_pfp FROM user WHERE id = ?;")
    .bind(user_id.user_id).fetch_one(db.as_ref()).await, HttpResponse::InternalServerError().finish());

    let image:String = try_or!(row.try_get("user_pfp"), HttpResponse::InternalServerError().finish());
    let user_name:String = try_or!(row.try_get("user_name"), HttpResponse::InternalServerError().finish());
    let user_email:String = try_or!(row.try_get("user_email"), HttpResponse::InternalServerError().finish());

    let base64_img = general_purpose::STANDARD.encode(&image);
    let data_url = format!("data:image/png;base64,{}", base64_img);

    return HttpResponse::Ok().json( &UserDetails{
        username: user_name,
        user_email: user_email,
        user_pfp: data_url
    });
}

#[post("/logout")]
pub async fn logout(db: web::Data<Pool<Sqlite>>, req: HttpRequest) -> HttpResponse {
    dbg!("Loging out - backend");
    let auth_header = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };
    let user_id = get_userid_from_jwt(&auth_header).await;
    if verify_token(db.clone(), auth_header).await {
       let _ = try_or!(sqlx::query("UPDATE user SET (user_refresh_token, user_access_token) = (?, ?) WHERE id = ?")
        .bind::<Option<String>>(None).bind::<Option<String>>(None)
        .bind(user_id)
        .execute(db.as_ref()).await, HttpResponse::InternalServerError().finish());
    }
    return HttpResponse::Ok().into();
}