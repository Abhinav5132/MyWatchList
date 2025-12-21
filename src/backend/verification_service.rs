use std::pin::Pin;

use jsonwebtoken::{DecodingKey, Validation, decode};

pub use crate::backend::*;

pub trait TokenVerifier: Sync + Send{
    fn verify_token<'a>(&'a self, token: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>;
    fn get_userid_from_jwt<'b>(&'b self ,token: &'b str) -> Pin<Box<dyn Future<Output = i64> + Send + 'b >>;
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

    fn get_userid_from_jwt<'b>(&'b self,token: &'b str) -> Pin<Box<dyn Future<Output = i64> + Send + 'b>> {
        Box::pin( async move{
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
                return c.claims.sub;
                }
                Err(e) =>{
                    dbg!(e);
                    return -1;
                }
            }}
        )
    }
}

pub struct MockVerificationService{
    pub should_return: bool
}

impl TokenVerifier for MockVerificationService {
    fn verify_token<'a>(&'a self, token: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send +'a>> {
        Box::pin(async move {return self.should_return})
    }

    fn get_userid_from_jwt<'b>(&'b self,token: &'b str) -> Pin<Box<dyn Future<Output = i64> + Send + 'b>> { // always return 1 cuz test user should always be on 1
        Box::pin(async move {
            return 1;
        })
    }
}