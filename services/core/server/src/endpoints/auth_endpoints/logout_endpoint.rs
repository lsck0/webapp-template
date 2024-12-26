use axum::response::IntoResponse;
use errors::{ServerResult, UserError};
use models::session_model::SessionInvalidationReason;

use crate::dtos::{DTO, session_dto::SessionDTO};

pub const LOGOUT_ENDPOINT: &str = "/api/auth/logout";

/// Logout a user.
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = LOGOUT_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
pub async fn logout_handler(session: SessionDTO) -> ServerResult<impl IntoResponse> {
    session.to_model()?.close(SessionInvalidationReason::UserLogout)?;

    return Ok(());
}
