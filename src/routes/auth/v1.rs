use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use serde::Serialize;

use crate::{
    db::{self, user::Role},
    error::AppError,
    middleware::auth::generate_token,
    routes::auth::{LoginRequest, LoginRfidRequest},
    state::AppState,
};

#[derive(Serialize)]
struct LoginResponseV1 {
    #[serde(rename = "accessToken")]
    access_token: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(login))
        .route("/rfid", post(rfid))
}

async fn login(State(state): State<AppState>, body: Json<LoginRequest>) -> impl IntoResponse {
    let logged_in_from_rv_terminal = body
        .rv_terminal_secret
        .as_deref()
        .map_or(false, |s| s == state.config.rv_terminal_secret);

    let user = db::user::find_user_by_username(&body.username, &state.database).await?;

    if user.role == Role::Inactive {
        return Err(AppError::NotAuthorized);
    }

    let verification = match bcrypt::verify(&body.password, &user.password_hash) {
        Ok(v) => v,
        Err(_) => {
            return Err(AppError::InvalidCredentials(
                "Invalid username or password".to_string(),
            ));
        }
    };

    if !verification {
        return Err(AppError::InvalidCredentials(
            "Invalid username or password".to_string(),
        ));
    }

    tracing::info!("User {} logged in with role {:?}", user.username, user.role);
    Ok(Json(LoginResponseV1 {
        access_token: generate_token(
            user.id,
            logged_in_from_rv_terminal,
            &state.config.jwt_secret,
            state.config.jwt_expiry,
        )?,
    }))
}

async fn rfid(State(state): State<AppState>, body: Json<LoginRfidRequest>) -> impl IntoResponse {
    let logged_in_from_rv_terminal = match body.rv_terminal_secret.as_deref() {
        Some(s) => s == state.config.rv_terminal_secret,
        None => {
            tracing::warn!("Rfid login failed, rv_terminal_secret not included");
            return Err(AppError::NotAuthorized);
        }
    };

    if !logged_in_from_rv_terminal {
        tracing::warn!("Rfid login failed, rv_terminal_secret not included");
        return Err(AppError::NotAuthorized);
    }

    let user = match db::user::find_by_rfid(&body.rfid, &state.config, &state.database).await? {
        Some(u) => u,
        None => {
            tracing::warn!("Failed to login with rfid");
            return Err(AppError::InvalidCredentials("Invalid rfid".to_string()));
        }
    };

    if user.role == Role::Inactive {
        tracing::warn!(
            "User {} is inactive, inactive user cannot login",
            user.username
        );
        return Err(AppError::NotAuthorized);
    }

    tracing::info!("User {} logged in with role {:?}", user.username, user.role);
    Ok(Json(LoginResponseV1 {
        access_token: generate_token(
            user.id,
            logged_in_from_rv_terminal,
            &state.config.jwt_secret,
            state.config.jwt_expiry,
        )?,
    }))
}
