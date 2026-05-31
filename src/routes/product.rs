use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, product::Product},
    state::AppState,
};

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Serialize)]
struct ProductResponse {
    product: Product,
}

#[derive(Serialize)]
struct ProductsResponse {
    products: Vec<Product>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(search))
        .route("/", get(get_products))
        .route("/{barcode}", get(get_product_by_barcode))
}

async fn search(State(state): State<AppState>, body: Json<SearchRequest>) -> impl IntoResponse {
    match db::product::search_products(&body.query, &state.database).await {
        Ok(products) => Ok(Json(ProductsResponse { products })),
        Err(e) => Err(e),
    }
}

async fn get_products(State(state): State<AppState>) -> impl IntoResponse {
    match db::product::get_products(&state.database).await {
        Ok(products) => Ok(Json(ProductsResponse { products })),
        Err(e) => Err(e),
    }
}

async fn get_product_by_barcode(
    State(state): State<AppState>,
    Path(barcode): Path<String>,
) -> impl IntoResponse {
    match db::product::find_by_barcode(&barcode, &state.database).await {
        Ok(product) => Ok(Json(ProductResponse { product })),
        Err(e) => Err(e),
    }
}
