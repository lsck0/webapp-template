use std::collections::HashMap;

use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Path},
};
use errors::ServerError;
use http::request::Parts;

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
        let params: Path<HashMap<String, String>> = parts.extract().await.map_err(|_| ServerError::MissingVersion)?;

        let version = params.get("version").ok_or(ServerError::MissingVersion)?;

        return match version.as_str() {
            "v1" => Ok(Version::V1),
            "v2" => Ok(Version::V2),
            "v3" => Ok(Version::V3),
            v => Err(ServerError::UnknownVersion(v.to_string())),
        };
    }
}
