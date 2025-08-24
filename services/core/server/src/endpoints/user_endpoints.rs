use axum::{Json, Router, extract::Query, middleware, response::IntoResponse, routing::get};
use errors::{ServerResult, UserError};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::{dtos::user_dto::UserDTO, middlewares::authentication_middleware::authentication_middleware};

pub const USER_ENDPOINT: &str = "/api/user";

pub fn user_router() -> Router {
    Router::new().nest(
        "/user",
        Router::new().route(
            "/",
            get(get_user_handler)
                .put(update_user_handler)
                .delete(delete_user_handler)
                .layer(middleware::from_fn(authentication_middleware)),
        ),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum GetUserRequest {
    Current {},
}

/// Get User Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Query Parameters:
///
/// {} - Current User.
#[utoipa::path(
    get,
    path = USER_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok.", body = Vec<UserDTO>),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn get_user_handler(parameters: Query<GetUserRequest>, user: UserDTO) -> ServerResult<impl IntoResponse> {
    match parameters.0 {
        GetUserRequest::Current {} => {
            return Ok(Json(vec![user]));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum UpdateUserRequest {
    Default,
}

/// Update User Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Request Body:
///
/// Not Implemented.
#[utoipa::path(
    put,
    path = USER_ENDPOINT,
    security(("token" = [])),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Ok.", body = Vec<UserDTO>),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn update_user_handler(Json(_parameters): Json<UpdateUserRequest>) -> ServerResult<impl IntoResponse> {
    return Ok(Json("Not Implemented."));
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum DeleteUserRequest {
    Default {},
}

/// Delete User Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Query Parameters:
///
/// Not Implemented.
#[utoipa::path(
    delete,
    path = USER_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn delete_user_handler(_parameters: Query<DeleteUserRequest>) -> ServerResult<impl IntoResponse> {
    return Ok(Json("Not Implemented."));
}
