use axum::{body::Body, extract::Request, middleware::Next, response::IntoResponse};
use errors::{ServerError, ServerResult};

use crate::dtos::Model;

pub async fn authentication_middleware(mut request: Request<Body>, next: Next) -> ServerResult<impl IntoResponse> {
    let provided_access_token =
        auth::extract_token_from_headers(request.headers()).ok_or(ServerError::Unauthenticated)?;

    let session = auth::authenticate(&provided_access_token)?;

    request.extensions_mut().insert(session.to_dto()?);

    return Ok(next.run(request).await);
}
