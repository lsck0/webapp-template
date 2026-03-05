use axum::{Json, Router, extract::Query, middleware, response::IntoResponse, routing::get};
use errors::{ServerError, ServerResult, UserError, Validate, bail};
use models::models::{permissions::Permissions, post_model::PostModel};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::ws_endpoints::WsNotification;
use crate::{
    dtos::{
        DTO, Model, ModelVectorExt,
        post_dto::{NewPostDTO, PostDTO},
        user_dto::UserDTO,
    },
    middlewares::authentication_middleware::authentication_middleware,
};

pub const POST_ENDPOINT: &str = "/api/post";

pub fn post_router() -> Router {
    Router::new().nest(
        "/post",
        Router::new().route(
            "/",
            get(get_post_handler)
                .post(create_post_handler)
                .put(update_post_handler)
                .delete(delete_post_handler)
                .layer(middleware::from_fn(authentication_middleware)),
        ),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum GetPostRequest {
    All {},
    ById {
        #[schema(value_type = String)]
        id: Uuid,
    },
}

/// Get Post Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Query Parameters:
///
/// {}     - Get all posts.
///
/// { id } - Get post by id.
#[utoipa::path(
    get,
    path = POST_ENDPOINT,
    security(("token" = [])),
    params(
        ("id" = Option<i32>, Query, description = "Post ID"),
    ),
    responses(
        (status = 200, description = "Ok.", body = Vec<PostDTO>),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn get_post_handler(parameters: Query<GetPostRequest>) -> ServerResult<impl IntoResponse> {
    match parameters.0 {
        GetPostRequest::All {} => {
            let posts = PostModel::get_all()?.to_dtos()?;

            return Ok(Json(posts));
        }
        GetPostRequest::ById { id } => {
            let Some(post) = PostModel::find_by_id(id)? else {
                bail!(ServerError::NonExistentId(id.to_string()));
            };

            let post = post.to_dto()?;

            return Ok(Json(vec![post]));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum CreatePostRequest {
    Default(NewPostDTO),
}

/// Create Post Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Request Body:
///
/// NewPostDTO - Create a new post from the NewPostDTO.
#[utoipa::path(
    post,
    path = POST_ENDPOINT,
    security(("token" = [])),
    request_body = CreatePostRequest,
    responses(
        (status = 200, description = "Ok.", body = Vec<PostDTO>),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn create_post_handler(Json(parameters): Json<CreatePostRequest>) -> ServerResult<impl IntoResponse> {
    match parameters {
        CreatePostRequest::Default(new_post) => {
            new_post.validate()?;

            let post = PostModel::new(new_post.to_model()?)?.to_dto()?;

            WsNotification::UpdatePosts.send().await;

            return Ok(Json(vec![post]));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum UpdatePostRequest {
    Default(PostDTO),
}

/// Update Post Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Request Body:
///
/// PostDTO - Update a post from the PostDTO.
#[utoipa::path(
    put,
    path = POST_ENDPOINT,
    security(("token" = [])),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "Ok.", body = Vec<PostDTO>),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn update_post_handler(Json(parameters): Json<UpdatePostRequest>) -> ServerResult<impl IntoResponse> {
    match parameters {
        UpdatePostRequest::Default(post) => {
            post.validate()?;

            let post = post.to_model()?.persist()?.to_dto()?;

            WsNotification::UpdatePosts.send().await;

            return Ok(Json(vec![post]));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(untagged)]
#[ts(export)]
pub enum DeletePostRequest {
    Default {
        #[schema(value_type = String)]
        id: Uuid,
    },
}

/// Delete Post Request.
///
/// Auth: Required
/// Permissions: None
///
/// ## Query Parameters:
///
/// { id } - Delete post by id.
#[utoipa::path(
    delete,
    path = POST_ENDPOINT,
    security(("token" = [])),
    params(
        ("id" = Option<i32>, Query, description = "Post ID"),
    ),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn delete_post_handler(
    parameters: Query<DeletePostRequest>,
    user: UserDTO,
) -> ServerResult<impl IntoResponse> {
    match parameters.0 {
        DeletePostRequest::Default { id } => {
            let Some(post) = PostModel::find_by_id(id)? else {
                bail!(ServerError::NonExistentId(id.to_string()));
            };

            if post.author != user.id {
                auth::require_permission(user.id, Permissions::CanDeleteAnyPost)?;
            }

            post.delete()?;

            WsNotification::UpdatePosts.send().await;

            return Ok(());
        }
    }
}
