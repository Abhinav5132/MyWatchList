pub use crate::*;
use serde_json::json;
use actix_web::HttpResponse;

#[derive(Deserialize)]
pub struct LoginStruct {
    username: String,
    password: String
}

#[post("/login")]
pub async fn login(db: web::Data<Pool<Sqlite>>, credentials: web::Json<LoginStruct>)-> HttpResponse{
    let username:&String = &credentials.username;
    let password = &credentials.password;

    let row =sqlx::query("SELECT user_password, id FROM user WHERE user_name = ?").bind(username).fetch_one(db.as_ref()).await; 

    match row {
        Ok(row_exist) =>{
            // if password verify = true return token with status message login sucessfull
            let hash_pwd:String = match row_exist.try_get("user_password"){
                Ok(pwd) => pwd ,
                Err(_)=>{
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "Internal error missing password hash"
                    }));
                }
            };
            
            match verify_pwd(&password, &hash_pwd) {
                Ok(_)=>{
                    let user_id = row_exist.try_get("id").unwrap_or(-1);
                    let token = match generate_token(user_id).await{
                        Ok(tok)=> tok,
                        Err(_)=> return HttpResponse::InternalServerError().json(serde_json::json!({
                                "Status": "Unable to login"
                                }))
                    };

                    let query = sqlx::query("
                    UPDATE user SET user_token = ? WHERE id = ?;
                    ").bind(&token).bind(user_id).execute(db.as_ref()).await;

                    match query {
                        Ok(_)=>{
                            HttpResponse::Ok() // this needs to also set the token in the database 
                            .insert_header(("Authorization", format!("Bearer {token}")))
                            .json(json!({
                                "Status": "Login successful"
                            }))
                        }

                        Err(_)=>{
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "Status": "Unable to login"
                                }))
                        }
                    }

                    
                }

                Err(_)=>{
                    //else no token and password is wrong
                    return HttpResponse::Unauthorized().json(serde_json::json!({
                        "Status": "Incorrect password",
                    }));
                }
            }

        }

        Err(sqlx::Error::RowNotFound) =>{
            // return no token, return status message that username is invalid
            return HttpResponse::NotFound().json(serde_json::json!({
                        "Status": "Invalid username",
                    }));
        }

        Err(_) => {
            // return unable to login due to an internal error
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "Status": "Unable to login"
            }));
        }
    }
}