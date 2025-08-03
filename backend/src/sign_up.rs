use actix_web::HttpResponse;

pub use crate::*;
use authenticate::pwd_to_hash;
use serde_json::json;
//implement profile pic later
#[derive(Deserialize)]
pub struct SignUpStruct{
    user_name: String,
    user_password: String,
    user_email: String,

}
#[post("/Signup")]
pub async fn sign_up_fn(db: web::Data<Pool<Sqlite>>, credentials: web::Json<SignUpStruct>) -> HttpResponse{
    let entered_pwd  = &credentials.user_password;
    let hashed_pwd = match pwd_to_hash(entered_pwd){
        Ok(pwd)=> pwd,
        Err(_)=>{
            dbg!("Unable to login 1");
            return HttpResponse::InternalServerError().json(json!({
                        "status": "Internal error converting pwd to hash"
                    }));
        }
    };
    dbg!(&credentials.user_name);
    dbg!(&entered_pwd);
    let user_id = match sqlx::query("INSERT INTO user (user_name, user_email, user_password) VALUES (?,?,?);")
    .bind(&credentials.user_name)
    .bind(&credentials.user_email)
    .bind(hashed_pwd)
    .execute(db.as_ref()).await
    {
        Ok(rows) => rows.last_insert_rowid(),
        Err(e)=> {
            dbg!(e);
            return HttpResponse::InternalServerError().json(json!({
                        "status": "Internal error"
                    }));
        }
    };

    let token = match generate_token(user_id).await {
        Ok(tok) => tok,
        Err(_)=>{
            dbg!("Unable to login 3");
            return HttpResponse::InternalServerError().json(json!({
                        "status": "Internal error generating token"
                    }));
        }
    };

    let query = sqlx::query("
        UPDATE user SET user_token = ? WHERE id = ?;
        ").bind(&token).bind(user_id).execute(db.as_ref()).await;

    match query {
        Ok(_)=>{
            dbg!("login successful 4");
            HttpResponse::Ok() 
            .insert_header(("Authorization", format!("Bearer {token}")))
            .json(json!({
                "Status": "Login successful"
            }))
        }

        Err(_)=>{
            dbg!("Unable to login");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "Status": "Unable to login"
                }))
        }
    }

}