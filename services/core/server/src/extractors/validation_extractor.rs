use axum::{
    Json,
    body::Body,
    extract::{FromRequest, Request},
};
use errors::{ServerError, Validate};
use serde::de::DeserializeOwned;

pub struct ValidatedBody<T>(pub T);
pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedBody<T>
where
    S: Send + Sync,
    T: Validate + DeserializeOwned,
{
    type Rejection = ServerError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(ServerError::from)?;

        value.validate()?;

        return Ok(ValidatedBody(value));
    }
}

impl<S, T> FromRequest<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: Validate + DeserializeOwned,
{
    type Rejection = ServerError;

    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
        let query = req.uri().query().unwrap_or("");
        let value = serde_qs::from_str::<T>(query)?;

        value.validate()?;

        return Ok(ValidatedQuery(value));
    }
}
