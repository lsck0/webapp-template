use axum::{Json, Router, extract::Query, middleware, response::IntoResponse, routing::get};
use errors::{ServerResult, UserError};
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
            let sessions = auth::get_all_sessions(session.user.id)?.to_dtos()?;

            return Ok(Json(sessions));
        }
        GetSessionRequest::Default { tag: Valid } => {
            let sessions = auth::get_valid_sessions(session.user.id)?.to_dtos()?;

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
    session: SessionDTO,
    Json(parameters): Json<CloseSessionRequest>,
) -> ServerResult<impl IntoResponse> {
    match parameters {
        CloseSessionRequest::All {} => {
            auth::close_all_sessions(session.user.id)?;
        }
        CloseSessionRequest::Id { id } => {
            auth::close_session(session.user.id, Uuid::parse_str(&id)?)?;
        }
    }

    return Ok(());
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
    match parameters.0 {
        DeleteSessionRequest::All {} => {
            auth::delete_all_sessions(session.user.id)?;
        }
        DeleteSessionRequest::Id { id } => {
            auth::delete_session(session.user.id, Uuid::parse_str(&id)?)?;
        }
    }

    return Ok(());
}
