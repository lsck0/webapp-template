use axum::{Json, response::IntoResponse};
use crypto::HashedPassword;
use errors::{FieldError, FieldErrorReason, ServerResult, UserError, bail, body_error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::dtos::{DTO, session_dto::SessionDTO};

pub const PW_CHANGE_ENDPOINT: &str = "/api/auth/password-change";

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct PasswordChangeInfo {
    pub old_password: String,
    pub new_password: String,
    pub otp: Option<String>,
}

/// Change the password of user belonging to the session.
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = PW_CHANGE_ENDPOINT,
    request_body = PasswordChangeInfo,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
pub async fn password_change_handler(
    session: SessionDTO,
    Json(pw_change_info): Json<PasswordChangeInfo>,
) -> ServerResult<impl IntoResponse> {
    let mut errors = vec![];

    // check the old password
    let mut user = session.user.to_model()?;

    let password_correct = HashedPassword::parse(&user.password_hash)?.check(&pw_change_info.old_password);

    if !password_correct {
        errors.push(FieldError {
            field: String::from("old_password"),
            reason: FieldErrorReason::InvalidCredentials,
        });

        bail!(body_error!(errors));
    }

    // TODO: check otp

    // create the new password and update the user
    user.password_hash = HashedPassword::new(&pw_change_info.new_password)?.consume();
    user.persist()?;

    return Ok(());
}
