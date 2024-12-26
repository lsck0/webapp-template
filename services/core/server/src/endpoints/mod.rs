pub mod auth_endpoints;
pub mod health_endpoints;
pub mod post_endpoints;
pub mod session_endpoints;
pub mod user_endpoints;
pub mod ws_endpoints;

use axum::{Router, http::Uri, response::IntoResponse, routing::get};
use errors::ServerError;
use ts_rs::TS;

/// API Endpoints.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, TS)]
#[ts(export)]
pub enum Endpoints {
    #[ts(rename = "/api/health")]
    HEALTH,

    #[ts(rename = "/api/ws")]
    WS,

    #[ts(rename = "/api/auth/invite")]
    INVITE,
    #[ts(rename = "/api/auth/register")]
    REGISTER,
    #[ts(rename = "/api/auth/login")]
    LOGIN,
    #[ts(rename = "/api/auth/logout")]
    LOGOUT,
    #[ts(rename = "/api/auth/refresh")]
    REFRESH,
    #[ts(rename = "/api/auth/password-change")]
    PASSWORD_CHANGE,
    #[ts(rename = "/api/auth/otp/enable")]
    OTP_ENABLE,
    #[ts(rename = "/api/auth/otp/disable")]
    OTP_DISABLE,
    #[ts(rename = "/api/auth/otp/validate")]
    OTP_VALIDATE,
    #[ts(rename = "/api/auth/otp/backup-codes")]
    OTP_BACKUP_CODES,

    #[ts(rename = "/api/user")]
    USER,

    #[ts(rename = "/api/session")]
    SESSION,

    #[ts(rename = "/api/post")]
    POST,
}

pub fn endpoint_router() -> Router {
    async fn api_endpoint_not_found(uri: Uri) -> impl IntoResponse {
        return ServerError::ApiEndpointNotFound(uri.path().to_string());
    }

    return Router::new().nest(
        "/api",
        Router::new()
            .merge(auth_endpoints::auth_router())
            .merge(health_endpoints::health_router())
            .merge(post_endpoints::post_router())
            .merge(session_endpoints::session_router())
            .merge(user_endpoints::user_router())
            .merge(ws_endpoints::ws_router())
            .route("/{*uri}", get(api_endpoint_not_found)),
    );
}
