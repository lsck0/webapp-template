use axum::{Json, Router, extract::Query, middleware, response::IntoResponse, routing::get};
use errors::{ServerError, ServerResult, UserError, bail};
use models::models::session_model::{SessionInvalidationReason, SessionModel};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    dtos::{ModelVectorExt, session_dto::SessionDTO},
    middlewares::authentication_middleware::authentication_middleware,
};

pub const SESSION_ENDPOINT: &str = "/api/session";

pub fn session_router() -> Router {
    Router::new().nest(
        "/session",
        Router::new().route(
            "/",
            get(get_session_handler)
                .put(close_session_handler)
                .delete(delete_session_handler)
                .layer(middleware::from_fn(authentication_middleware)),
        ),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum GetSessionRequest {
    Default { tag: GetSessionTag },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub enum GetSessionTag {
    All,
    Valid,
    Current,
}

/// Get Session Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Query Parameters:
///
/// { tag } - Get All/Valid/Current sessions.
#[utoipa::path(
    get,
    path = SESSION_ENDPOINT,
    security(("token" = [])),
    params(
        ("tag" = Option<GetSessionTag>, Query),
    ),
    responses(
        (status = 200, description = "Ok.", body = Vec<SessionDTO>),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn get_session_handler(
    parameters: Query<GetSessionRequest>,
    session: SessionDTO,
) -> ServerResult<impl IntoResponse> {
    use GetSessionTag::*;
    match parameters.0 {
        GetSessionRequest::Default { tag: All } => {
            let sessions = SessionModel::find_by_user(session.user.id)?.to_dtos()?;

            return Ok(Json(sessions));
        }
        GetSessionRequest::Default { tag: Valid } => {
            let sessions = SessionModel::find_valid_by_user(session.user.id)?.to_dtos()?;

            return Ok(Json(sessions));
        }
        GetSessionRequest::Default { tag: Current } => {
            return Ok(Json(vec![session]));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum CloseSessionRequest {
    All {},
    Id { id: String },
}

/// Close Session Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Request Body:
///
/// {}     - Close all sessions.
///
/// { id } - Close session by id.
#[utoipa::path(
    put,
    path = SESSION_ENDPOINT,
    security(("token" = [])),
    request_body = CloseSessionRequest,
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn close_session_handler(
    parameters: Query<CloseSessionRequest>,
    session: SessionDTO,
) -> ServerResult<impl IntoResponse> {
    use CloseSessionRequest::*;
    match parameters.0 {
        All {} => {
            SessionModel::close_all_for_user(session.user.id, SessionInvalidationReason::UserClosed)?;

            return Ok(());
        }
        Id { id } => {
            let Some(session_to_close) = SessionModel::find_by_id(Uuid::parse_str(&id)?)? else {
                bail!(ServerError::NonExistentId(id));
            };

            if session_to_close.user_id != session.user.id {
                bail!(ServerError::Unauthorized);
            }

            session_to_close.close(SessionInvalidationReason::UserClosed)?;

            return Ok(());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum DeleteSessionRequest {
    All {},
    Id { id: String },
}

/// Delete Session Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Query Parameters:
///
/// {}     - Delete all sessions.
///
/// { id } - Delete session by id.
#[utoipa::path(
    delete,
    path = SESSION_ENDPOINT,
    security(("token" = [])),
    params(
        ("id" = Option<String>, Query),
    ),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn delete_session_handler(
    parameters: Query<DeleteSessionRequest>,
    session: SessionDTO,
) -> ServerResult<impl IntoResponse> {
    use DeleteSessionRequest::*;
    match parameters.0 {
        All {} => {
            SessionModel::delete_all_for_user(session.user.id)?;

            return Ok(());
        }
        Id { id } => {
            let Some(session_to_delete) = SessionModel::find_by_id(Uuid::parse_str(&id)?)? else {
                bail!(ServerError::NonExistentId(id));
            };

            if session_to_delete.user_id != session.user.id {
                bail!(ServerError::Unauthorized);
            }

            session_to_delete.delete()?;

            return Ok(());
        }
    }
}
