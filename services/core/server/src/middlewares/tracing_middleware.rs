//! TODO: look into the use of auth in here

use std::{sync::OnceLock, time::Duration};

use axum::{
    Router,
    body::Body,
    extract::{MatchedPath, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use config::ServerConfig;
use crypto::JWTToken;
use http::{Method, header};
use http_body_util::BodyExt;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{Span, error, info, info_span};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Fields that should be redacted from logs.
const REDACTED_FIELDS: [&str; 4] = ["password", "otp", "access_token", "session_token"];
const REDACTED_PATHS: [&str; 1] = ["/api/metrics"];

/// Keep track if this module has been initialized. The subscriber panics if it is initialized
/// multiple times. (like in tests)
static TRACING_INITIALIZED: OnceLock<bool> = OnceLock::new();

/// Registers stdio and file logging subscriber.
pub(crate) fn initialize_tracing() {
    match TRACING_INITIALIZED.get() {
        Some(_) => return,
        None => TRACING_INITIALIZED.set(true).expect("unreachable"),
    }

    let log_level = match ServerConfig::get().dev {
        true => "trace",
        false => "info",
    };

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| format!("server={log_level}").into()))
        .with(fmt::layer().json().with_writer(std::io::stdout))
        .init();
}

/// Applies the tracing layer to the router.
pub(crate) fn add_tracing_layer(router: Router) -> Router {
    router
        .layer(middleware::from_fn(response_body_extractor_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let path = request.extensions().get::<MatchedPath>().map(MatchedPath::as_str);

                    // we get this from nginx via the header, not from the actual connection
                    let addr = request
                        .headers()
                        .get("X-Real-IP")
                        .and_then(|header| header.to_str().ok())
                        .unwrap_or("unknown");

                    let body = request.extensions().get::<RequestBody>().map(|body| body.0.clone());

                    let session = request
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
                        .and_then(|token| JWTToken::parse(&token).ok())
                        .and_then(|jwt_token| jwt_token.decode().ok())
                        .and_then(|payload| payload.subject_as_uuid().ok());

                    info_span!(
                        "Request",
                        from = addr,
                        verb = ?request.method(),
                        path,
                        ?session,
                        body,
                    )
                })
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    // skip some paths (like /metrics)
                    let path = response.extensions().get::<MatchedPath>().map(MatchedPath::as_str);
                    dbg!(path);

                    if REDACTED_PATHS.contains(&path.unwrap_or_default()) {
                        return;
                    }

                    let latency_micros = latency.as_micros();

                    let body = response.extensions().get::<ResponseBody>().map(|body| body.0.clone());

                    let (time, unit) = match latency_micros {
                        0..1_000 => (latency_micros as f64, "µs"),
                        1_000..1_000_000 => (latency.as_millis() as f64, "ms"),
                        _ => (latency.as_secs_f64(), "s"),
                    };

                    if let Some(body) = body
                        && !body.is_empty()
                    {
                        info!("Responded with body {body:?} ({}) in {time}{unit}.", response.status());
                    } else {
                        info!("Responded with status {} in {time}{unit}.", response.status());
                    }
                })
                .on_failure(|error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                    error!("Request failed. {}.", error);
                }),
        )
        .layer(middleware::from_fn(request_body_extractor_middleware))
}

// We have to take the request, buffer it (aka wait for it to fully arrive) and then insert a copy
// into the extensions to be logged. Same with the response.

#[derive(Debug, Clone)]
struct RequestBody(String);

#[derive(Debug, Clone)]
struct ResponseBody(String);

async fn request_body_extractor_middleware(mut request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    // Some requests don't have a body
    if [Method::POST, Method::PUT].contains(request.method()) {
        let (parts, body) = request.into_parts();

        let (body, body_string) = buffer_body(body).await?;

        request = Request::from_parts(parts, body);
        request.extensions_mut().insert(RequestBody(body_string));
    }

    let response = next.run(request).await;

    return Ok(response);
}

async fn response_body_extractor_middleware(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    let response = next.run(request).await;

    let (parts, body) = response.into_parts();

    let (body, body_string) = buffer_body(body).await?;

    let mut response = Response::from_parts(parts, body);
    response.extensions_mut().insert(ResponseBody(body_string));

    return Ok(response);
}

// body -> (body, string representation)
async fn buffer_body(body: Body) -> Result<(Body, String), Response> {
    // this won't work if the body is an long running stream
    let raw_body_bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
        .to_bytes();

    let mut body_string = String::from_utf8(raw_body_bytes.to_vec().clone())
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    // we dont always respond with JSON, but if we do censor some fields
    if let Ok(mut body_json) = serde_json::from_str::<serde_json::Value>(&body_string) {
        match body_json {
            serde_json::Value::Object(ref mut obj) => {
                for field in REDACTED_FIELDS.into_iter() {
                    if let Some(value) = obj.get_mut(field) {
                        *value = serde_json::Value::String(String::from("[REDACTED]"));
                    }
                }
            }
            serde_json::Value::Array(ref mut array) => {
                for item in array.iter_mut() {
                    if let serde_json::Value::Object(obj) = item {
                        for field in REDACTED_FIELDS.into_iter() {
                            if let Some(value) = obj.get_mut(field) {
                                *value = serde_json::Value::String(String::from("[REDACTED]"));
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        body_string = serde_json::to_string(&body_json)
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error serializing JSON: {}", err),
                )
                    .into_response()
            })?
            .replace("\"", "'"); // prettier logging
    }

    return Ok((Body::from(raw_body_bytes), body_string));
}
