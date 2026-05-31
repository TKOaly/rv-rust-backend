use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};

use crate::{db::{self, product::Product}, state::AppState};

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Serialize)]
struct SearchResponse {
    products: Vec<Product>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/leaderboard", get(search))
}

async fn search(State(state): State<AppState>, body: Json<SearchRequest>) -> impl IntoResponse {
    match db::product::search_products(&body.query, &state.database).await {
        Ok(products) => Ok(Json(SearchResponse { products })),
        Err(e) => Err(e),
    }
}

