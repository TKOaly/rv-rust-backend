use crate::{
    config::AppConfig,
    db::{self, user::Role},
    error::{AppError, Result},
    state::AppState,
};
use axum::{
    body::Body,
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,
    pub logged_in_from_rv_terminal: bool,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i32,
    pub logged_in_from_rv_terminal: bool,
}

pub fn generate_token(
    user_id: i32,
    logged_in_from_rv_terminal: bool,
    secret: &str,
    expiry: i64,
) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expiry)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        logged_in_from_rv_terminal,
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
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::InvalidToken,
        _ => AppError::InvalidToken,
    })
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str> {
    headers
        .get("Authorization")
        .ok_or(AppError::BadRequest)?
        .to_str()
        .map_err(|_| AppError::BadRequest)?
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::BadRequest)
}

pub async fn jwt_middleware(
    State(config): State<AppConfig>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let token = extract_bearer_token(req.headers())?;
    let claims = validate_token(token, &config.jwt_secret)?;

    req.extensions_mut().insert(AuthUser {
        user_id: claims.sub.parse()?,
        logged_in_from_rv_terminal: claims.logged_in_from_rv_terminal,
    });

    Ok(next.run(req).await)
}

#[derive(Deserialize)]
struct BodyWithRvTerminalSecret {
    #[serde(rename = "rvTerminalSecret")]
    rv_terminal_secret: Option<String>,
}

pub async fn require_rv_terminal(
    State(config): State<AppConfig>,
    req: Request,
    next: Next,
) -> Result<Response> {
    let (parts, body) = req.into_parts();

    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| AppError::BadRequest)?;

    let logged_in_from_rv_terminal = serde_json::from_slice::<BodyWithRvTerminalSecret>(&bytes)
        .ok()
        .and_then(|body| body.rv_terminal_secret)
        .map(|secret| secret == config.rv_terminal_secret)
        .unwrap_or(false);

    if !logged_in_from_rv_terminal {
        return Err(AppError::Forbidden);
    }

    let req = Request::from_parts(parts, Body::from(bytes));

    Ok(next.run(req).await)
}

pub async fn require_role(
    State(state): State<AppState>,
    role: Role,
    req: Request,
    next: Next,
) -> Result<Response> {
    let auth = match req.extensions().get::<AuthUser>() {
        Some(a) => a,
        None => {
            return Err(AppError::Internal(anyhow::format_err!(
                "Jwt middleware is not called"
            )));
        }
    };

    let user = db::user::find_user_by_id(auth.user_id, &state.database).await?;

    if user.role != role {
        return Err(AppError::NotAuthorized);
    }

    Ok(next.run(req).await)
}

pub async fn require_active_account(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response> {
    let auth = match req.extensions().get::<AuthUser>() {
        Some(a) => a,
        None => {
            return Err(AppError::Internal(anyhow::format_err!(
                "Jwt middleware is not called"
            )));
        }
    };

    let user = db::user::find_user_by_id(auth.user_id, &state.database).await?;

    if user.role != Role::Inactive {
        return Err(AppError::NotAuthorized);
    }

    Ok(next.run(req).await)
}
