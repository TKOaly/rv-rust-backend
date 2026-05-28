use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::{
    db::{self, user::Leaderboard},
    state::AppState,
};

#[derive(Serialize)]
struct LeaderboardResponse {
    leaderboard: Vec<Leaderboard>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/leaderboard", get(leaderboard))
}

async fn leaderboard(State(state): State<AppState>) -> impl IntoResponse {
    match db::user::leaderboard(&state.database).await {
        Ok(leaderboard) => Ok(Json(LeaderboardResponse { leaderboard })),
        Err(e) => Err(e),
    }
}
