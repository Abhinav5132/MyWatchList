use std::pin::Pin;

use jsonwebtoken::{DecodingKey, Validation, decode};

pub use crate::backend::*;

pub trait TokenVerifier: Sync + Send{
    fn verify_token<'a>(&'a self, token: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>;
}

pub struct VerificationService{
    pub db: Data<Pool<Sqlite>>
}

impl TokenVerifier for VerificationService{
    fn verify_token<'a>(&'a self, token: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send +'a>> {
        Box::pin(async move { 
            let token = match token.strip_prefix("Bearer ") {
            Some(t) => t,
            None => return false, 
        };
        
        
        dbg!("verifying",token);
        dotenvy::dotenv().ok();
        let secret = std::env::var("JWT_ACCESS_KEY").expect("Secret key must be set");
        let validation = Validation::default();
        let claims = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation
        ){
            Ok(c)=> {
                c.claims
            }
            Err(e) =>{
                dbg!(e);
                dbg!("Error decoding the key");
                return false;
            }
        };

        if let Ok(row) = sqlx::query("SELECT user_access_token FROM user WHERE id = ?").bind(claims.sub)
        .fetch_one(self.db.as_ref()).await {
            let db_tk:Result<String, sqlx::Error> = row.try_get("user_access_token");
            if let Ok(db_token) = db_tk{
                return db_token == token && claims.exp > chrono::Utc::now().timestamp() as usize;
            }
        }
        false
        })
    }
}

pub struct MockVerificationService{
    pub should_return: bool
}

impl TokenVerifier for MockVerificationService {
    fn verify_token<'a>(&'a self, token: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send +'a>> {
        Box::pin(async move {return self.should_return})
    }
}