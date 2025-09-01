use std::path::{self, Path};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::{OsRng};

use crate::*;

#[derive(Serialize, Deserialize)]
pub struct Claims{
    pub sub: i64,
    pub exp: usize
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

pub async fn generate_token(user_id: i64) ->Result<String, jsonwebtoken::errors::Error>{
    let expiery = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize;
    let claims = Claims{
        sub: user_id,
        exp: expiery
    };
    dotenvy::dotenv().ok();

    let secret = std::env::var("JWT_KEY").expect("Secret key must be set");
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub async fn get_userid_from_jwt(token: &str) -> i64 {
    dotenvy::dotenv().ok();
    let secret = std::env::var("JWT_KEY").expect("Secret key must be set");
    let validation = Validation::default();
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
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

pub async fn verify_token(mut token: &str) -> bool {
    token = token.strip_prefix("Bearer ").unwrap();
    if !token.starts_with("Bearer "){
        return false;
    }
    
    dotenvy::dotenv().ok();
    let secret = std::env::var("JWT_KEY").expect("Secret key must be set");
    let validation = Validation::default();
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation
    ){
        Ok(c)=> {
            let current = chrono::Utc::now().timestamp() as usize;
            let expiry = c.claims.exp;
            if expiry < current {
                return false;
            } else{
                return true;
            }
        }
        Err(e) =>{
            dbg!(e);
            return false;
        }
    }
}