use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::Deserialize;

use crate::{db, email, error::AppError, state::AppState};

#[derive(Deserialize)]
struct TempPasswordRequest {
    #[serde(rename = "userId")]
    user_id: i32,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/temp_password", post(generate_temp_password))
}

async fn generate_temp_password(
    State(state): State<AppState>,
    Json(body): Json<TempPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = db::user::find_user_by_id(body.user_id, &state.database).await?;
    let temp_password = db::user::create_temp_password(user.id, &state.database).await?;

    tracing::info!("Temporary password generated for user {}", user.username);

    let smtp_response = email::send_temp_password(&user, &temp_password, &state.mailer).await?;

    if !smtp_response.is_positive() {
        return Err(AppError::Internal(anyhow::format_err!(
            "Email was not sent: {}",
            smtp_response.first_word().unwrap_or("")
        )));
    }

    tracing::info!("Sent email: {}", smtp_response.first_word().unwrap_or(""));

    Ok((
        StatusCode::CREATED,
        "Temporary password created successfully.",
    ))
}
