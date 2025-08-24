use std::net::SocketAddr;

use axum::{Json, extract::ConnectInfo, response::IntoResponse};
use chrono::{Duration, Utc};
use config::ServerConfig;
use crypto::{HashedPassword, JWTToken};
use errors::{FieldError, FieldErrorReason, ServerResult, UserError, bail, body_error, user_error};
use http::{HeaderMap, header::USER_AGENT};
use ipnetwork::IpNetwork;
use models::models::{
    login_attempt_model::LoginAttemptModel,
    login_restriction_model::{LoginRestrictionModel, NewLoginRestrictionModel},
    session_model::{NewSessionModel, SessionModel},
    user_model::UserModel,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::dtos::{DTO, Model, session_dto::SessionDTO};

pub const LOGIN_ENDPOINT: &str = "/api/auth/login";

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct LoginInfo {
    pub name: String,
    pub password: String,
    pub otp: Option<String>,
}

/// Login a user.
///
/// Auth: Not Required
/// Permissions: None
#[utoipa::path(
    post,
    path = LOGIN_ENDPOINT,
    request_body = LoginInfo,
    responses(
        (status = 200, description = "Ok.", body = SessionDTO),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
pub async fn login_handler(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(login_info): Json<LoginInfo>,
) -> ServerResult<impl IntoResponse> {
    let mut errors = vec![];

    // get the user
    let Some(user) = UserModel::find_by_name(&login_info.name)? else {
        push_invalid_login_errors(&mut errors);
        bail!(body_error!(errors));
    };

    // check if the user has a login restriction
    if let Some(restriction) = LoginRestrictionModel::find_by_user_id(user.id)?
        && restriction.restricted_until > Utc::now()
    {
        push_invalid_login_errors(&mut errors);
        bail!(body_error!(errors));
    }

    // check if password is correct
    if !HashedPassword::parse(&user.password_hash)?.check(&login_info.password) {
        register_failed_login_attempt(&user.id)?;
        push_invalid_login_errors(&mut errors);
        bail!(body_error!(errors));
    }

    // TODO: check otp

    // check number of open sessions
    let open_sessions = SessionModel::find_valid_by_user(user.id)?.len();
    if open_sessions >= ServerConfig::get().max_open_sessions {
        bail!(user_error!(TooManyOpenSessions));
    }

    // create the session
    let mut session = SessionModel::new(NewSessionModel {
        user_id: user.id,
        session_token: String::new(),
        access_token: String::new(),
        ip_address: IpNetwork::from(addr.ip()),
        user_agent: headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map_or(String::from("unknown"), |v| v.to_string()),
    })?
    .to_dto()?;

    // create the tokens and update the session
    session.session_token = JWTToken::new_session_token(&session.id).consume();
    session.access_token = JWTToken::new_access_token(&session.id).consume();
    let session = session.to_model()?.persist()?.to_dto()?;

    // delete failed login attempts
    LoginAttemptModel::delete_all_for_user(&user.id)?;

    return Ok(Json(session));
}

fn push_invalid_login_errors(errors: &mut Vec<FieldError>) {
    errors.push(FieldError {
        field: String::from("name"),
        reason: FieldErrorReason::InvalidCredentials,
    });
    errors.push(FieldError {
        field: String::from("password"),
        reason: FieldErrorReason::InvalidCredentials,
    });
    errors.push(FieldError {
        field: String::from("otp"),
        reason: FieldErrorReason::InvalidCredentials,
    });
}

fn register_failed_login_attempt(user_id: &Uuid) -> ServerResult<()> {
    let past_attempt = LoginAttemptModel::find_past_attempt(user_id)?;

    // increment the attempt count or create a new one, restrict if necessary
    let max_attempts = ServerConfig::get().max_login_attempts;
    match past_attempt {
        Some(mut attempt) => {
            attempt.failed_counter += 1;
            attempt.last_attempt = Utc::now();
            let attempt = attempt.persist()?;

            if attempt.failed_counter + 1 >= max_attempts {
                let restriction_time = ServerConfig::get().login_restriction_time;

                LoginRestrictionModel::new(NewLoginRestrictionModel {
                    user_id: *user_id,
                    restricted_until: Utc::now() + Duration::minutes(restriction_time),
                })?;
            }
        }
        None => {
            LoginAttemptModel::new(*user_id)?;
        }
    }

    return Ok(());
}
