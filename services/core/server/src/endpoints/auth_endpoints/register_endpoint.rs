use axum::{Json, response::IntoResponse};
use chrono::Utc;
use crypto::HashedPassword;
use errors::{FieldError, FieldErrorReason, ServerResult, UserError, bail, body_error, ensure, validate_string_length};
use models::models::{
    invite_model::InviteModel,
    user_model::{NewUserModel, UserModel},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

pub const REGISTER_ENDPOINT: &str = "/api/auth/register";

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct RegisterInfo {
    pub name: String,
    pub password: String,
    pub invite: String,
}

/// Register a new user.
///
/// Auth: Not Required
/// Permissions: None
#[utoipa::path(
    post,
    path = REGISTER_ENDPOINT,
    request_body = RegisterInfo,
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
pub async fn register_handler(Json(register_info): Json<RegisterInfo>) -> ServerResult<impl IntoResponse> {
    // validate the input
    let mut errors = vec![];

    validate_string_length!(errors, register_info, name, 4, 32);
    validate_string_length!(errors, register_info, password, 8, 32);

    if UserModel::find_by_name(&register_info.name)?.is_some() {
        errors.push(FieldError {
            field: String::from("name"),
            reason: FieldErrorReason::UserAlreadyExists(register_info.name.clone()),
        });
    }

    ensure!(errors.is_empty(), body_error!(errors));

    // check the invite
    let Some(invite) = InviteModel::find_by_token(&register_info.invite)? else {
        errors.push(FieldError {
            field: String::from("invite"),
            reason: FieldErrorReason::InvalidInviteToken,
        });
        bail!(body_error!(errors));
    };
    if invite.used || invite.expires_at < Utc::now() {
        errors.push(FieldError {
            field: String::from("invite"),
            reason: FieldErrorReason::ExpiredInviteToken,
        });
        bail!(body_error!(errors));
    }

    // create the user
    let password_hash = HashedPassword::new(&register_info.password)?.consume();
    let user = UserModel::new(NewUserModel {
        name: register_info.name,
        password_hash,
    })?;

    // mark the invite as used
    invite.use_invite(user.id)?;

    return Ok(());
}
