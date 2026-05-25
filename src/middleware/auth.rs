use crate::{config::AppConfig, error::Result};
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing Authorization header")]
    MissingHeader,
    #[error("Invalid Authorization header format (expected: Bearer <token>)")]
    InvalidHeaderFormat,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Insufficient permissions")]
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
}

pub fn generate_token(user_id: usize, secret: String, expiry: i64) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expiry)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired.into(),
        _ => AuthError::InvalidToken(e.to_string()).into(),
    })
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str> {
    headers
        .get("Authorization")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaderFormat)?
        .strip_prefix("Bearer ")
        .ok_or_else(|| AuthError::InvalidHeaderFormat.into())
}

pub async fn jwt_middleware(
    State(config): State<AppConfig>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let token = extract_bearer_token(req.headers())?;
    let claims = validate_token(token, &config.jwt_secret)?;

    req.extensions_mut().insert(AuthUser {
        user_id: claims.sub,
    });

    Ok(next.run(req).await)
}

