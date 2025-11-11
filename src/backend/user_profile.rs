use actix_web::{web::{Data, Json}, HttpRequest, HttpResponse};
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

#[derive(Serialize, Deserialize)]
pub struct ChangeUsername{
    user_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePassword{
    pwd: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeEmail{
    email: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePfp{
    pfp: String,
}

#[post("/get_user_details")]
pub async fn get_user_details(db: web::Data<Pool<Sqlite>>, user_id: Json<UserId>) -> HttpResponse{
    let row = try_or!(sqlx::query("SELECT user_name, user_email, user_pfp FROM user WHERE id = ?;")
    .bind(user_id.user_id).fetch_one(db.as_ref()).await, HttpResponse::InternalServerError().finish());

    let image:String = try_or!(row.try_get("user_pfp"), HttpResponse::InternalServerError().finish());
    let user_name:String = try_or!(row.try_get("user_name"), HttpResponse::InternalServerError().finish());
    let user_email:String = try_or!(row.try_get("user_email"), HttpResponse::InternalServerError().finish());


    return HttpResponse::Ok().json( &UserDetails{
        username: user_name,
        user_email: user_email,
        user_pfp: image
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
#[post("/change_username")]
pub async fn change_username(db: web::Data<Pool<Sqlite>>, req: HttpRequest, username: Json<ChangeUsername>) ->HttpResponse {
    let token = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };
    let user_id = get_userid_from_jwt(token).await;
    
    if verify_token(db.clone(), token).await{
        let _row = try_or!(
            sqlx::query("UPDATE user SET user_name = ? WHERE id = ?")
            .bind(&username.user_name).bind(user_id).execute(db.as_ref()).await, HttpResponse::InternalServerError().finish()
        );

        return HttpResponse::Ok().into();
    }
    HttpResponse::Unauthorized().into()
}

#[post("/change_password")]
pub async fn change_password(db: web::Data<Pool<Sqlite>>, req: HttpRequest, username: Json<ChangePassword>) -> HttpResponse{
    dbg!("changing_password");
    let token = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };
    let user_id = get_userid_from_jwt(token).await;
    let pwd_hash = try_or!(pwd_to_hash(&username.pwd), HttpResponse::Unauthorized().into());
    if verify_token(db.clone(), token).await{
        let _row =try_or!(
            sqlx::query("UPDATE user SET user_password = ? WHERE id = ?")
            .bind(pwd_hash).bind(user_id).execute(db.as_ref()).await, HttpResponse::InternalServerError().finish()
        );
        return HttpResponse::Ok().into();
    }
    return HttpResponse::Unauthorized().into()
}

#[post("/change_email")]
pub async fn change_email(db: web::Data<Pool<Sqlite>>, req: HttpRequest, username: Json<ChangeEmail>) -> HttpResponse{ // add email verification later
    let token = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };
    let user_id = get_userid_from_jwt(token).await;
    if verify_token(db.clone(), token).await{
        let _row =try_or!(
            sqlx::query("UPDATE user SET user_email = ? WHERE id = ?")
            .bind(&username.email).bind(user_id).execute(db.as_ref()).await, HttpResponse::InternalServerError().finish()
        );
        return HttpResponse::Ok().into()
    }
    return HttpResponse::Unauthorized().into()
}

#[post("/delete_user")]
pub async fn delete_user(db: web::Data<Pool<Sqlite>>, req: HttpRequest) -> HttpResponse {
    
    let token = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };
    let user_id = get_userid_from_jwt(token).await;
    if verify_token(db.clone(), token).await{ 
        let _row = try_or!(
            sqlx::query("DELETE user WHERE id = ?").bind(user_id).execute(db.as_ref()).await,
            HttpResponse::InternalServerError().finish()
        );
        return HttpResponse::Ok().into()
    }
    return HttpResponse::Ok().into()
}

#[post("/change_pfp")]
pub async fn change_pfp(db: Data<Pool<Sqlite>>, req: HttpRequest, new_pfp: Json<ChangePfp>) -> HttpResponse {
    let token = match req.headers().get("Authorization") {
        Some(a) => {
            a.to_str().unwrap_or("")
        }
        None =>{
            return HttpResponse::Unauthorized().into();
        }
    };
    let user_id = get_userid_from_jwt(token).await;
    if verify_token(db.clone(), token).await{
        let _ = try_or!(
            sqlx::query("UPDATE user SET user_pfp = ? WHERE id = ?")
            .bind(&new_pfp.pfp).bind(user_id).execute(db.as_ref()).await, 
            HttpResponse::InternalServerError().finish() 
        );

        return HttpResponse::Ok().into();
     }
    return HttpResponse::Unauthorized().into();
}
// these two are long term 
pub async fn export_all_my_lists() {
todo!()
}

pub async fn export_one_list() {
todo!()
}