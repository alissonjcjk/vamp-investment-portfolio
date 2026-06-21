use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing Authorization")]
    MissingAuthorization,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Asset does not exist")]
    AssetDoesNotExist,
    #[error("User does not exist")]
    UserDoesNotExist,
    #[error("Username already taken")]
    UsernameTaken,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[error(transparent)]
    Jwt(#[from] jwt_simple::Error),
}

#[derive(Serialize)]
struct ErrorResponse { error: String }

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::UsernameTaken | Self::MissingAuthorization => StatusCode::BAD_REQUEST,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AssetDoesNotExist | Self::UserDoesNotExist => StatusCode::NOT_FOUND,
            Self::Database(_) | Self::Template(_) | Self::Jwt(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(ErrorResponse { error: self.to_string() })).into_response()
    }
}
