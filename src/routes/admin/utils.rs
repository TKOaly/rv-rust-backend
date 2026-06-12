use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};

use crate::{db, middleware::auth::AuthUser, routes::user::UserResponse, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/getUserByUsername/{username}", get(get_by_username))
        .route("/getUserByEmail/{email}", get(get_by_email))
        .route("/getUserByFullName/{fullname}", get(get_by_fullname))
}

async fn get_by_username(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    match db::user::find_user_by_username(&username, &state.database).await {
        Ok(user) => {
            tracing::info!(
                "User with id {} fetched fetched user with username {} as admin",
                auth.user_id,
                username
            );
            (StatusCode::OK, Json(UserResponse::from(user))).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_by_email(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(email): Path<String>,
) -> impl IntoResponse {
    match db::user::find_user_by_email(&email, &state.database).await {
        Ok(user) => {
            tracing::info!(
                "User with id {} fetched fetched user with email {} as admin",
                auth.user_id,
                email
            );
            (StatusCode::OK, Json(UserResponse::from(user))).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_by_fullname(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fullname): Path<String>,
) -> impl IntoResponse {
    match db::user::find_user_by_fullname(&fullname, &state.database).await {
        Ok(user) => {
            tracing::info!(
                "User with id {} fetched users with full name {} as admin",
                auth.user_id,
                fullname
            );
            (StatusCode::OK, Json(UserResponse::from(user))).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
