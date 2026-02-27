use axum::extract::FromRequestParts;
use errors::ServerError;
use http::request::Parts;

use crate::dtos::{session_dto::SessionDTO, user_dto::UserDTO};

impl<S> FromRequestParts<S> for UserDTO
where
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        return parts
            .extensions
            .get::<SessionDTO>()
            .ok_or(ServerError::Unauthenticated)
            .cloned()
            .map(|session| session.user);
    }
}
