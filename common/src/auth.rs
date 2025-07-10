//! Authentication and authorization utilities

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub roles: Vec<String>,
}

impl Claims {
    pub fn new(user_id: String, roles: Vec<String>, expires_in_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);
        
        Claims {
            sub: user_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            roles,
        }
    }
}

pub fn generate_jwt(claims: &Claims, secret: &str) -> Result<String> {
    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    
    encode(&header, claims, &encoding_key)
        .map_err(|e| Error::Auth(format!("Failed to generate JWT: {}", e)))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| Error::Auth(format!("Invalid JWT: {}", e)))
}

pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| Error::Auth(format!("Failed to hash password: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash)
        .map_err(|e| Error::Auth(format!("Failed to verify password: {}", e)))
}