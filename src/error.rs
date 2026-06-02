use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Not authorized")]
    NotAuthorized,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Forbidden")]
    Forbidden,
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Identifier taken {0}")]
    IdentifierTaken(String),
    #[error("Bad request")]
    BadRequest,
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.to_string()),
            AppError::NotAuthorized => (StatusCode::FORBIDDEN, "not_authorized", self.to_string()),
            AppError::InsufficientFunds => (
                StatusCode::FORBIDDEN,
                "insufficient_funds",
                self.to_string(),
            ),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            AppError::InvalidCredentials(msg) => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                msg.to_string(),
            ),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "Invalid authorization token".to_string(),
            ),
            AppError::IdentifierTaken(msg) => {
                (StatusCode::CONFLICT, "identifier_taken", msg.to_string())
            }
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "bad_request", self.to_string()),
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".to_string(),
                )
            }
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AppError::Internal(anyhow::anyhow!(error))
    }
}

impl From<sea_orm::error::DbErr> for AppError {
    fn from(error: sea_orm::error::DbErr) -> Self {
        AppError::Internal(anyhow::anyhow!(error))
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(error: std::num::ParseIntError) -> Self {
        AppError::Internal(anyhow::anyhow!(error))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(error: bcrypt::BcryptError) -> Self {
        AppError::Internal(anyhow::anyhow!(error))
    }
}

impl From<hex::FromHexError> for AppError {
    fn from(error: hex::FromHexError) -> Self {
        AppError::Internal(anyhow::anyhow!(error))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
