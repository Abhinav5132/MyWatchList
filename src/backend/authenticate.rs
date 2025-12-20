use actix_web::{web::{Data, Json}, HttpRequest, HttpResponse};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};use rand_core::{OsRng};
use crate::{backend::{login::LoginStruct, sign_up::AuthResponse, *}, try_or};
#[derive(Serialize, Deserialize)]
pub struct Claims{
    pub sub: i64,
    pub exp: usize,
    pub iat: usize,
}


#[derive(Serialize, Deserialize)]
pub struct IssueNewAccess{
    pub access_token: String,
    pub expiry: u64
}

pub fn pwd_to_hash(pwd: &str)-> Result<String, argon2::password_hash::Error>{
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash= argon2.hash_password(pwd.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
    
}

pub fn verify_pwd(entered_pwd: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    let if_valid = argon2.verify_password(entered_pwd.as_bytes(), &parsed_hash);
    Ok(if_valid.is_ok())

}

pub async fn generate_access_token(user_id: i64) ->Result<String, jsonwebtoken::errors::Error>{
    let expiery = (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize;
    let claims = Claims{
        sub: user_id,
        exp: expiery,
        iat: chrono::Utc::now().timestamp() as usize
    };

    let secret = std::env::var("JWT_ACCESS_KEY").expect("Secret key must be set");
    dbg!("generating access token");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )

}

pub async fn generate_refresh_token(user_id: i64) ->Result<String, jsonwebtoken::errors::Error>{
    let expiery = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize;
    let claims = Claims{
        sub: user_id,
        exp: expiery,
        iat: chrono::Utc::now().timestamp() as usize
    };

    let secret = std::env::var("JWT_REFRESH_KEY").expect("Secret key must be set");
    dbg!("generating refresh token");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}


pub async fn get_userid_from_jwt(token: &str) -> i64 {
    dbg!("getting userid from jwt");
    let token = match token.strip_prefix("Bearer ") {
        Some(t) => t,
        None => return -1, 
    };
    dotenvy::dotenv().ok();
    let secret = std::env::var("JWT_ACCESS_KEY").expect("Secret key must be set");
    let validation = Validation::default();
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation
    ){
        Ok(c)=> {
            c.claims.sub
        }
        Err(e) =>{
            dbg!(e);
            return -1;
        }
    }
}

#[post("/issue_new_access")]
pub async fn issue_new_access_token(db: Data<Pool<Sqlite>>, refresh_token: Json<AuthResponse>) -> HttpResponse {
    dotenvy::dotenv().ok();
    match sqlx::query("SELECT id 
    FROM user 
    WHERE user_refresh_token = ?;
    ").bind(&refresh_token.refresh_token).fetch_one(db.as_ref()).await {
        Ok(row) => {
            let user_id:i64 = match row.try_get("id"){
                Ok(u) => u,
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::Unauthorized().into()}
            };
            let access_token = match generate_access_token(user_id).await {
                Ok(token) => {
                    let _ = match sqlx::query("
                    UPDATE user SET user_access_token = ? WHERE id = ?;
                    ").bind(&token).bind(user_id).execute(db.as_ref()).await{
                        Ok(a) => a,
                        Err(e) => {
                            dbg!(e);
                            return HttpResponse::Unauthorized().into();
                        }
                    };
                    token
                },
                Err(e) => {
                    dbg!(e);
                    return HttpResponse::Unauthorized().into();
                }
            };
            return HttpResponse::Ok().json(IssueNewAccess{
                access_token: access_token,
                expiry: (chrono::Utc::now() + chrono::Duration::minutes(3)).timestamp() as u64
            });
        }   
        Err(e)=> {
            dbg!(e);
            return HttpResponse::Unauthorized().into();
        }
    }
}   

#[post("/verify_password")]
pub async fn verify_entered_password(db: Data<Pool<Sqlite>>, req: HttpRequest, user: Json<LoginStruct>) -> HttpResponse {
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
        sqlx::query("SELECT user_password FROM user WHERE id = ?").bind(user_id).fetch_one(db.as_ref()).await,
        HttpResponse::InternalServerError().finish()
    );

    let hash:&str = try_or!(row.try_get("user_password"), HttpResponse::InternalServerError().finish());

    let verification = try_or!(verify_pwd(&user.password, hash), HttpResponse::Unauthorized().into());

    if verification {
        return HttpResponse::Ok().into();
    }
    
    HttpResponse::Unauthorized().into()
}