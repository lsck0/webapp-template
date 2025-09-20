//! Error handling for the application.
//! There are two types of errors: "server errors" and "user errors". Server errors are errors that can occur _by using
//! the server_, hence are problems of the developer.
//!
//! User errors are errors that can occur _by using the application_, hence are problems of the user and thus have to be
//! handled differently to allow proper feedback to the user.
//!
//! User errors are usually sent with code 422 (unprocessable entity aka semantic mistake) as [UserError].

pub mod macros;

use axum::{Json, extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

/// Result type with error set to [ServerError].
pub type ServerResult<T> = Result<T, ServerError>;

pub trait Validate {
    fn validate(&self) -> ServerResult<()>;
}

/// All possible errors that can occur.
#[derive(Debug, Error)]
pub enum ServerError {
    // *
    // User errors.
    // *
    #[error("User error")]
    UserError(UserError),

    // *
    // Programmer Errors.
    // *
    #[error("Non-existent id: {0}")]
    NonExistentId(String),

    // Auth errors.
    #[error("Unauthenticated")]
    Unauthenticated,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Expired token")]
    ExpiredToken,

    // Routing errors.
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("API endpoint not found: {0}")]
    ApiEndpointNotFound(String),

    // Database errors.
    #[error("Diesel pool exhaustion")]
    DBPoolExhausted,

    // Fallback error.
    #[error("Generic error: {0}")]
    Generic(String),

    // *
    // Forwards of other errors.
    // *
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),
    #[error(transparent)]
    TokenError(#[from] jsonwebtoken::errors::Error),
    #[error("Argon2 hashing error")]
    HashingError,
    #[error(transparent)]
    UuidError(#[from] uuid::Error),
}

/// User errors.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub enum UserError {
    BodyError(Vec<FieldError>),
    TooManyOpenSessions,
    Generic(String),
}

/// A field error.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct FieldError {
    pub field: String,
    pub reason: FieldErrorReason,
}

/// Reasons for why fields are invalid.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub enum FieldErrorReason {
    InvalidRange { min: i32, max: i32 },
    InvalidCredentials,
    ExpiredInviteToken,
    InvalidInviteToken,
    UserAlreadyExists(String),
}

/// Converting the error into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        use ServerError::*;

        match self {
            UserError(err) => (StatusCode::UNPROCESSABLE_ENTITY, Json(err)).into_response(),

            NonExistentId(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),

            Unauthenticated => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            Unauthorized => (StatusCode::FORBIDDEN, self.to_string()).into_response(),
            InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            ExpiredToken => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),

            FileNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            ApiEndpointNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),

            DBPoolExhausted => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()).into_response(),

            Generic(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),

            JsonRejection(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response(),
            DieselError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
            TokenError(_) => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            HashingError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
            UuidError(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response(),
        }
    }
}
