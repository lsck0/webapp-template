use axum::response::IntoResponse;
use chrono::Duration;
use errors::{ServerResult, UserError};
use models::models::invite_model::InviteModel;

use crate::dtos::user_dto::UserDTO;

pub const INVITE_ENDPOINT: &str = "/api/auth/invite";

/// Create an invite to allow a new user to register.
///
/// Auth: Required
/// Permissions: >=Staff
#[utoipa::path(
    get,
    path = INVITE_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok.", body = String),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
pub async fn invite_handler(user: UserDTO) -> ServerResult<impl IntoResponse> {
    // if user.permissions < Permissions::Staff {
    //     bail!(ServerError::Unauthorized);
    // }

    let invite = InviteModel::new(user.id, Duration::days(1))?;

    return Ok(invite.token);
}
