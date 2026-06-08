use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, category::Category},
    error::{AppError, Result},
    middleware::auth::AuthUser,
    state::AppState,
};

#[derive(Deserialize)]
struct CreateCategoryRequest {
    description: String,
}

#[derive(Serialize)]
struct CategoryResponse {
    category: Category,
}

#[derive(Serialize)]
struct CategoriesResponse {
    categories: Vec<Category>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_categories))
        .route("/", post(create_category))
        .route("/{category_id}", get(get_category))
}

async fn get_categories(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    match db::category::get_all_categories(&state.database).await {
        Ok(categories) => {
            tracing::info!("User with id {} fetched categories as admin", auth.user_id);
            Ok(Json(CategoriesResponse { categories }))
        }
        Err(e) => Err(e),
    }
}

async fn create_category(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    body: Json<CreateCategoryRequest>,
) -> impl IntoResponse {
    match db::category::insert_category(&body.description, &state.database).await {
        Ok(category) => {
            tracing::info!(
                "User with id {} created new category {} as admin",
                auth.user_id,
                body.description
            );
            (StatusCode::CREATED, Json(category)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

async fn get_category(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(category_id): Path<i32>,
) -> Result<Json<CategoryResponse>> {
    let category = db::category::find_category_by_id(category_id, &state.database)
        .await?
        .ok_or_else(|| AppError::NotFound("Category does not exist".to_string()))?;

    tracing::info!("User with id {} fetched category as admin", auth.user_id);
    Ok(Json(CategoryResponse { category }))
}
