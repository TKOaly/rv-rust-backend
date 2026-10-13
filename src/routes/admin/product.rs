use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};

use crate::{
    db::{self},
    error::AppError,
    middleware::auth::AuthUser,
    routes::product::{get_product_by_barcode, get_products},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_products))
        .route("/{barcode}", get(get_product_by_barcode))
}
