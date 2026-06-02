use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};

use crate::{
    db::{self, user::InsertUserData},
    error::AppError,
    routes::user::UserResponse,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(register))
}

async fn register(State(state): State<AppState>, body: Json<InsertUserData>) -> impl IntoResponse {
    if db::user::find_user_by_username(&body.username, &state.database)
        .await
        .is_ok()
    {
        tracing::warn!(
            "Failed to register new user, username {} was already taken",
            &body.username
        );
        return AppError::IdentifierTaken("Username is already in use.".to_string())
            .into_response();
    }

    if db::user::find_user_by_email(&body.email, &state.database)
        .await
        .is_ok()
    {
        tracing::warn!(
            "Failed to register new user, email {} was already taken",
            &body.email
        );
        return AppError::IdentifierTaken("Email is already in use.".to_string()).into_response();
    }

    let user = match db::user::insert_user(body.0, &state.database).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    (StatusCode::CREATED, Json(UserResponse::from(user))).into_response()
}
