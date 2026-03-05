pub mod analytics_endpoints;
pub mod auth_endpoints;
pub mod health_endpoints;
pub mod post_endpoints;
pub mod session_endpoints;
pub mod user_endpoints;
pub mod ws_endpoints;

use axum::{Router, http::Uri, response::IntoResponse, routing::get};
use errors::ServerError;
use ts_rs::TS;

use crate::endpoints::{
    analytics_endpoints::ANALYTICS_ENDPOINT,
    auth_endpoints::{
        INVITE_ENDPOINT, LOGIN_ENDPOINT, LOGOUT_ENDPOINT, OTP_BACKUP_CODES_ENDPOINT, OTP_DISABLE_ENDPOINT,
        OTP_ENABLE_ENDPOINT, OTP_VALIDATE_ENDPOINT, PASSWORD_CHANGE_ENDPOINT, REFRESH_ENDPOINT, REGISTER_ENDPOINT,
    },
    health_endpoints::HEALTH_ENDPOINT,
    post_endpoints::POST_ENDPOINT,
    session_endpoints::SESSION_ENDPOINT,
    user_endpoints::USER_ENDPOINT,
    ws_endpoints::WEBSOCKET_ENDPOINT,
};

/// API Endpoints.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, TS)]
#[ts(export)]
pub enum Endpoints {
    #[ts(rename = HEALTH_ENDPOINT)]
    HEALTH,

    #[ts(rename = WEBSOCKET_ENDPOINT)]
    WS,

    #[ts(rename = ANALYTICS_ENDPOINT)]
    ANALYTICS,

    #[ts(rename = INVITE_ENDPOINT)]
    INVITE,
    #[ts(rename = REGISTER_ENDPOINT)]
    REGISTER,
    #[ts(rename = LOGIN_ENDPOINT)]
    LOGIN,
    #[ts(rename = LOGOUT_ENDPOINT)]
    LOGOUT,
    #[ts(rename = REFRESH_ENDPOINT)]
    REFRESH,
    #[ts(rename = PASSWORD_CHANGE_ENDPOINT)]
    PASSWORD_CHANGE,
    #[ts(rename = OTP_ENABLE_ENDPOINT)]
    OTP_ENABLE,
    #[ts(rename = OTP_DISABLE_ENDPOINT)]
    OTP_DISABLE,
    #[ts(rename = OTP_VALIDATE_ENDPOINT)]
    OTP_VALIDATE,
    #[ts(rename = OTP_BACKUP_CODES_ENDPOINT)]
    OTP_BACKUP_CODES,

    #[ts(rename = USER_ENDPOINT)]
    USER,

    #[ts(rename = SESSION_ENDPOINT)]
    SESSION,

    #[ts(rename = POST_ENDPOINT)]
    POST,
}

pub fn endpoint_router() -> Router {
    async fn api_endpoint_not_found(uri: Uri) -> impl IntoResponse {
        return ServerError::ApiEndpointNotFound(uri.path().to_string());
    }

    return Router::new().nest(
        "/api",
        Router::new()
            .merge(analytics_endpoints::analytics_router())
            .merge(auth_endpoints::auth_router())
            .merge(health_endpoints::health_router())
            .merge(post_endpoints::post_router())
            .merge(session_endpoints::session_router())
            .merge(user_endpoints::user_router())
            .merge(ws_endpoints::ws_router())
            .route("/{*uri}", get(api_endpoint_not_found)),
    );
}
