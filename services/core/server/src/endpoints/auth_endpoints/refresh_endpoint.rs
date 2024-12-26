use axum::{Json, response::IntoResponse};
use chrono::Utc;
use crypto::JWTToken;
use errors::{ServerError, ServerResult, UserError, bail};
use models::{models::session_model::SessionModel, session_model::SessionInvalidationReason};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::dtos::{Model, session_dto::SessionDTO};

pub const REFRESH_ENDPOINT: &str = "/api/auth/refresh";

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct SessionRefreshInfo {
    #[schema(value_type = String)]
    pub session_id: Uuid,
    pub session_token: String,
}

/// Refresh a session.
///
/// Auth: Not Required
/// Permissions: None
#[utoipa::path(
    post,
    path = REFRESH_ENDPOINT,
    request_body = SessionRefreshInfo,
    responses(
        (status = 200, description = "Ok.", body = SessionDTO),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
pub async fn refresh_handler(Json(session_refresh_info): Json<SessionRefreshInfo>) -> ServerResult<impl IntoResponse> {
    // check if the session is even valid
    let Some(mut session) = SessionModel::find_by_id(session_refresh_info.session_id)? else {
        bail!(ServerError::Unauthenticated);
    };

    if !session.valid {
        bail!(ServerError::Unauthenticated);
    }

    // check if the session token is still valid
    let session_token_payload = JWTToken::parse(&session_refresh_info.session_token).and_then(|token| token.decode());

    if matches!(session_token_payload, Err(ServerError::InvalidToken)) {
        bail!(ServerError::Unauthenticated);
    }

    if matches!(session_token_payload, Err(ServerError::ExpiredToken)) {
        session.close(SessionInvalidationReason::Expired)?;
        bail!(ServerError::ExpiredToken);
    }

    session_token_payload?;

    // susipicious activity, there should not be two valid session tokens
    if session.session_token != session_refresh_info.session_token {
        session.close(SessionInvalidationReason::SessionTokenLeak)?;
        bail!(ServerError::Unauthenticated);
    }

    // update the session
    session.session_token = JWTToken::new_session_token(&session.id).consume();
    session.access_token = JWTToken::new_access_token(&session.id).consume();
    session.last_used = Utc::now();
    let session = session.persist()?.to_dto()?;

    return Ok(Json(session));
}
