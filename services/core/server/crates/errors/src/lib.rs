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
    #[error("Account locked until {0}")]
    AccountLocked(String),
    #[error("Invalid OTP code")]
    InvalidOtp,
    #[error("OTP already enabled")]
    OtpAlreadyEnabled,
    #[error("OTP not enabled")]
    OtpNotEnabled,

    // Routing errors.
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("API endpoint not found: {0}")]
    ApiEndpointNotFound(String),
    #[error("Unknown version: {0}")]
    UnknownVersion(String),
    #[error("Version missing")]
    MissingVersion,
    #[error("Method not allowed")]
    MethodNotAllowed,

    // Rate limiting errors.
    #[error("Rate limit exceeded, retry after {0}s")]
    RateLimited(u64),

    // Database errors.
    #[error("Diesel pool exhaustion")]
    DBPoolExhausted,
    #[error("Database connection failed")]
    DBConnectionFailed,
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Conflict: resource already exists")]
    Conflict,

    // External service errors.
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Request timeout")]
    Timeout,

    // Payload errors.
    #[error("Payload too large")]
    PayloadTooLarge,
    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    // Fallback error.
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Generic error: {0}")]
    Generic(String),

    // *
    // Forwards of other errors.
    // *
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    QueryRejection(#[from] serde_qs::Error),
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
    AccountLocked(String),
    PasswordTooWeak(String),
    EmailNotVerified,
    InviteRequired,
    OtpAlreadyEnabled,
    OtpNotEnabled,
    OtpRequired,
    InvalidOtp,
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
    InvalidFormat,
    Required,
    InvalidCredentials,
    ExpiredInviteToken,
    InvalidInviteToken,
    UserAlreadyExists(String),
    EmailAlreadyInUse(String),
    InvalidEmail,
    PasswordMismatch,
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
            AccountLocked(_) => (StatusCode::FORBIDDEN, self.to_string()).into_response(),
            InvalidOtp => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            OtpAlreadyEnabled => (StatusCode::CONFLICT, self.to_string()).into_response(),
            OtpNotEnabled => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),

            FileNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            ApiEndpointNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            UnknownVersion(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            MissingVersion => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            MethodNotAllowed => (StatusCode::METHOD_NOT_ALLOWED, self.to_string()).into_response(),

            RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, self.to_string()).into_response(),

            DBPoolExhausted => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()).into_response(),
            DBConnectionFailed => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()).into_response(),
            TransactionFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
            Conflict => (StatusCode::CONFLICT, self.to_string()).into_response(),

            ServiceUnavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()).into_response(),
            Timeout => (StatusCode::GATEWAY_TIMEOUT, self.to_string()).into_response(),

            PayloadTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()).into_response(),
            UnsupportedMediaType(_) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string()).into_response(),

            Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
            Generic(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),

            JsonRejection(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response(),
            QueryRejection(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response(),
            DieselError(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal server error")).into_response(),
            TokenError(_) => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            HashingError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
            UuidError(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response(),
        }
    }
}
