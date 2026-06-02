use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        self,
        product::{Product, PurchaseEvent},
    },
    error::AppError,
    middleware::auth::AuthUser,
    state::AppState,
};

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Deserialize)]
struct PurchaseRequest {
    count: i32,
}

#[derive(Serialize)]
struct ProductResponse {
    product: Product,
}

#[derive(Serialize)]
struct ProductsResponse {
    products: Vec<Product>,
}

#[derive(Serialize)]
struct PurchaseResponse {
    #[serde(rename = "accountBalance")]
    balance: i32,
    #[serde(rename = "productStock")]
    product_stock: i32,
    purchases: Vec<PurchaseEvent>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_products))
        .route("/search", get(search))
        .route("/{barcode}", get(get_product_by_barcode))
        .route("/{barcode}/return", post(return_product))
        .route("/{barcode}/purchase", post(purchase_product))
}

async fn search(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    body: Json<SearchRequest>,
) -> impl IntoResponse {
    match db::product::search_products(&body.query, &state.database).await {
        Ok(products) => {
            tracing::info!(
                "User whit id {} searched for products with query: {}",
                auth.user_id,
                body.query
            );
            Ok(Json(ProductsResponse { products }))
        }
        Err(e) => Err(e),
    }
}

async fn get_products(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    match db::product::get_products(&state.database).await {
        Ok(products) => {
            tracing::info!("User whit id {} fetched products", auth.user_id);
            Ok(Json(ProductsResponse { products }))
        }
        Err(e) => Err(e),
    }
}

async fn get_product_by_barcode(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(barcode): Path<String>,
) -> impl IntoResponse {
    match db::product::find_by_barcode(&barcode, &state.database).await {
        Ok(product) => {
            tracing::info!("User with id {} fetched products", auth.user_id);
            Ok(Json(ProductResponse { product }))
        }
        Err(_) => {
            tracing::warn!(
                "User whit id {} tried to fetch unknown product {}",
                auth.user_id,
                barcode
            );
            Err(AppError::NotFound("Product not found".to_string()))
        }
    }
}

async fn return_product(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(barcode): Path<String>,
) -> impl IntoResponse {
    match db::product::return_purchase(&barcode, auth.user_id, &state.database).await {
        Ok(success) => {
            if success {
                tracing::info!(
                    "User with id {} returned product {} successfully",
                    auth.user_id,
                    barcode
                );
                return StatusCode::OK.into_response();
            } else {
                tracing::info!(
                    "User with id {} attempted to return a product {} unsuccessfully",
                    auth.user_id,
                    barcode
                );
                StatusCode::FORBIDDEN.into_response()
            }
        }
        Err(e) => e.into_response(),
    }
}

async fn purchase_product(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(barcode): Path<String>,
    body: Json<PurchaseRequest>,
) -> impl IntoResponse {
    let user = match db::user::find_user_by_id(auth.user_id, &state.database).await {
        Ok(u) => u,
        Err(e) => return Err(e),
    };

    let product = match db::product::find_by_barcode(&barcode, &state.database).await {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!(
                "User {} tried to purchase unknown product {}",
                user.username,
                barcode
            );
            return Err(AppError::NotFound("Product not found".to_string()));
        }
    };

    if product.sell_price <= 0 || user.saldo > product.sell_price * (body.count - 1) {
        let purchases =
            match db::product::record_purchase(&barcode, auth.user_id, body.count, &state.database)
                .await
            {
                Ok(p) => p,
                Err(e) => return Err(e),
            };

        tracing::info!(
            "User {} purchased {} x product {}",
            user.username,
            body.count,
            barcode
        );

        Ok(Json(PurchaseResponse {
            balance: user.saldo - body.count * product.sell_price,
            product_stock: product.stock - body.count,
            purchases: purchases,
        }))
    } else {
        tracing::warn!(
            "User {} tried to purchase {} x product {} but didn't have enough money.",
            user.username,
            body.count,
            barcode
        );
        Err(AppError::InsufficientFunds)
    }
}
