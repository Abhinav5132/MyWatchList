use jsonwebtoken::{encode, EncodingKey, Header};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::{OsRng};

use crate::*;
use crate::login::Claims;


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

pub async fn generate_token(user_id: i32) ->Result<String, jsonwebtoken::errors::Error>{
    let expiery = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize;
    let claims = Claims{
        sub: user_id,
        exp: expiery
    };
    let secret = std::env::var("JWT_KEY").expect("Secret key must be set");

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

