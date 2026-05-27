use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};

use crate::{db, state::AppState};

#[derive(Deserialize)]
struct UserExistsRequest {
    username: String,
}

#[derive(Serialize)]
struct UserExistsResponse {
    exists: bool,
}

pub fn public_routes() -> Router<AppState> {
    Router::new().route("/user_exists", post(user_exists))
}

async fn user_exists(
    State(state): State<AppState>,
    body: Json<UserExistsRequest>,
) -> Json<UserExistsResponse> {
    match db::user::find_user_by_username(&body.username, &state.database).await {
        Ok(_) => Json(UserExistsResponse { exists: true }),
        Err(_) => Json(UserExistsResponse { exists: false }),
    }
}
