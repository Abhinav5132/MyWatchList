
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;


pub fn pwd_to_has(pwd: &str)-> Result<String, argon2::password_hash::Error>{
    
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