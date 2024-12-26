use axum::{Json, Router, response::IntoResponse, routing::get};
use errors::ServerResult;

pub const HEALTH_ENDPOINT: &str = "/api/health";

pub fn health_router() -> Router {
    Router::new().nest("/health", Router::new().route("/", get(health)))
}

/// Check server health.
///
/// Auth: Not Required
/// Permissions: None
#[utoipa::path(
    get,
    path = HEALTH_ENDPOINT,
    responses(
        (status = 200, description = "Ok.", body = String),
    ),
)]
async fn health() -> ServerResult<impl IntoResponse> {
    return Ok(Json("healthy"));
}
