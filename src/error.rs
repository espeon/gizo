use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Failed to fetch URL: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Failed to process image")]
    ImageProcessingError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InvalidUrl => (StatusCode::BAD_REQUEST, "Invalid URL"),
            AppError::FetchError(_) => (StatusCode::BAD_REQUEST, "Failed to fetch URL"),
            AppError::ImageProcessingError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process image")
            } //AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        (status, message).into_response()
    }
}
