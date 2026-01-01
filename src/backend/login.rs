use crate::backend::sign_up::AuthResponse;
pub use crate::backend::*;
use actix_web::HttpResponse;
use serde_json::json;

#[derive(Deserialize)]
pub struct LoginStruct {
    pub username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login_fn(
    db: web::Data<Pool<Sqlite>>,
    credentials: web::Json<LoginStruct>,
) -> HttpResponse {
    let username: &String = &credentials.username;
    let password = &credentials.password;
    dbg!(&username);
    dbg!(&password);
    let row = sqlx::query("SELECT user_password, id FROM user WHERE user_name = ?")
        .bind(username)
        .fetch_one(db.as_ref())
        .await;

    match row {
        Ok(row_exist) => {
            // if password verify = true return token with status message login sucessfull
            let hash_pwd: String = match row_exist.try_get("user_password") {
                Ok(pwd) => pwd,
                Err(_) => {
                    dbg!("Internal error missing password hash");
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "Internal error missing password hash"
                    }));
                }
            };
            match verify_pwd(password, &hash_pwd) {
                Ok(verify_result) => {
                    if verify_result {
                        let user_id = row_exist.try_get("id").unwrap_or(-1);
                        let access_token = match generate_access_token(user_id).await {
                            Ok(tok) => tok,
                            Err(_) => {
                                dbg!("Unable to login");
                                return HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                    "Status": "Unable to login"
                                    }),
                                );
                            }
                        };
                        let refresh_token = match generate_refresh_token(user_id).await {
                            Ok(tok) => tok,
                            Err(_) => {
                                dbg!("Unable to login");
                                return HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                    "Status": "Unable to login"
                                    }),
                                );
                            }
                        };

                        let query = sqlx::query("
                        UPDATE user SET (user_access_token, user_refresh_token) = (?, ?) WHERE id = ?;
                        ").bind(&access_token).bind(&refresh_token).bind(user_id).execute(db.as_ref()).await;

                        match query {
                            Ok(_) => {
                                dbg!("Login successful");
                                return HttpResponse::Ok().json(AuthResponse {
                                    access_token: access_token,
                                    refresh_token: refresh_token,
                                    expires_in: (chrono::Utc::now() + chrono::Duration::minutes(3))
                                        .timestamp()
                                        as u64,
                                });
                            }

                            Err(e) => {
                                dbg!("Unable to login", e);
                                return HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                    "Status": "Unable to login"
                                    }),
                                );
                            }
                        }
                    }
                    dbg!("incorrect password");
                    HttpResponse::Unauthorized().json(serde_json::json!({
                        "Status": "Incorrect password",
                    }))
                }

                Err(_) => {
                    //else no token and password is wrong
                    dbg!("incorrect password");
                    HttpResponse::Unauthorized().json(serde_json::json!({
                        "Status": "Incorrect password",
                    }))
                }
            }
        }

        Err(sqlx::Error::RowNotFound) => {
            // return no token, return status message that username is invalid
            dbg!("Invalid username");
            HttpResponse::NotFound().json(serde_json::json!({
                "Status": "Invalid username",
            }))
        }

        Err(_) => {
            // return unable to login due to an internal error
            dbg!("Internal server error");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "Status": "Unable to login"
            }))
        }
    }
}
