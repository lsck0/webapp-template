use std::collections::HashMap;

use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Path},
    response::{IntoResponse, Response},
};
use errors::ServerError;
use http::{StatusCode, request::Parts};

#[derive(Debug)]
pub enum Version {
    V1,
    V2,
    V3,
}

impl<S> FromRequestParts<S> for Version
where
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params: Path<HashMap<String, String>> = parts.extract().await.map_err(IntoResponse::into_response)?;

        let version = params
            .get("version")
            .ok_or_else(|| (StatusCode::NOT_FOUND, "version param missing").into_response())?;

        return match version.as_str() {
            "v1" => Ok(Version::V1),
            "v2" => Ok(Version::V2),
            "v3" => Ok(Version::V3),
            v => Err(ServerError::UnknownVersion(v.to_string())),
        };
    }
}
