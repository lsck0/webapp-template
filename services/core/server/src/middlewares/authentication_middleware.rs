// TODO: refactor this with the new auth api
use axum::{body::Body, extract::Request, http::header, middleware::Next, response::IntoResponse};
use crypto::JWTToken;
use errors::{ServerError, ServerResult, bail, ensure};
use models::models::session_model::{SessionInvalidationReason, SessionModel};

use crate::dtos::Model;

pub async fn authentication_middleware(mut request: Request<Body>, next: Next) -> ServerResult<impl IntoResponse> {
    // get access token authorization header or bail
    let provided_access_token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer ").map(|token| token.to_string()))
        .or_else(|| {
            request
                .headers()
                .get(header::SEC_WEBSOCKET_PROTOCOL)
                .and_then(|ws_header| ws_header.to_str().ok())
                .map(|ws_token_str| ws_token_str.to_string())
        })
        .ok_or_else(|| ServerError::Unauthenticated)?;

    // check the session associated with the access token
    let session_id = JWTToken::parse(&provided_access_token)?.decode()?.subject_as_uuid()?;

    let Some(session) = SessionModel::find_by_id(session_id)? else {
        bail!(ServerError::Unauthenticated);
    };

    ensure!(session.valid, ServerError::Unauthenticated);

    // suspicious behavior, there should not be two valid access tokens
    if provided_access_token != session.access_token {
        session.close(SessionInvalidationReason::AccessTokenLeak)?;

        bail!(ServerError::Unauthenticated);
    }

    request.extensions_mut().insert(session.to_dto()?);

    return Ok(next.run(request).await);
}
