use axum::{Json, Router, response::IntoResponse, routing::post};
use chrono::{DateTime, Utc};
use errors::ServerResult;
use metrics::counter;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

pub const ANALYTICS_ENDPOINT: &str = "/api/analytics";

const MAX_EVENTS_PER_BATCH: usize = 64;
const MAX_EVENT_NAME_LEN: usize = 64;
const MAX_PROPERTY_KEY_LEN: usize = 32;
const MAX_PROPERTY_VALUE_LEN: usize = 128;
const MAX_PROPERTIES_PER_EVENT: usize = 8;

pub fn analytics_router() -> Router {
    Router::new().nest("/analytics", Router::new().route("/", post(collect_analytics_handler)))
}

/// A single client analytics event.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct AnalyticsEvent {
    /// Event name (e.g., "page_view", "click", "error").
    pub name: String,
    /// Optional key-value properties for context.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, String>,
}

/// Batch of analytics events from the client.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct AnalyticsRequest {
    pub events: Vec<AnalyticsEvent>,
}

/// A recorded analytics event with server-assigned timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct RecordedAnalyticsEvent {
    pub name: String,
    pub properties: std::collections::HashMap<String, String>,
    #[schema(value_type = String)]
    pub timestamp: DateTime<Utc>,
}

/// Collect client analytics.
///
/// Auth: Not Required
/// Permissions: None
///
/// Accepts a batch of analytics events from the client, stamps each with the
/// current server time, records them as Prometheus metrics, and returns the
/// recorded events.
#[utoipa::path(
    post,
    path = ANALYTICS_ENDPOINT,
    request_body = AnalyticsRequest,
    responses(
        (status = 200, description = "Events recorded.", body = Vec<RecordedAnalyticsEvent>),
        (status = 422, description = "Invalid request."),
    ),
)]
async fn collect_analytics_handler(Json(req): Json<AnalyticsRequest>) -> ServerResult<impl IntoResponse> {
    let events = &req.events;

    if events.len() > MAX_EVENTS_PER_BATCH {
        return Err(errors::ServerError::PayloadTooLarge);
    }

    let now = Utc::now();
    let mut recorded = Vec::with_capacity(events.len());

    for event in events {
        let name = sanitize_metric_label(&event.name, MAX_EVENT_NAME_LEN);

        counter!("client_event_total", "event" => name.clone()).increment(1);

        let sanitized_props: std::collections::HashMap<String, String> = event
            .properties
            .iter()
            .take(MAX_PROPERTIES_PER_EVENT)
            .map(|(k, v)| {
                (
                    sanitize_metric_label(k, MAX_PROPERTY_KEY_LEN),
                    sanitize_metric_label(v, MAX_PROPERTY_VALUE_LEN),
                )
            })
            .collect();

        recorded.push(RecordedAnalyticsEvent {
            name,
            properties: sanitized_props,
            timestamp: now,
        });
    }

    return Ok(Json(recorded));
}

/// Sanitize a string for use as a Prometheus metric label value.
/// Keeps only alphanumeric, underscore, hyphen, dot, slash, and space. Truncates to `max_len`.
fn sanitize_metric_label(input: &str, max_len: usize) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ' '))
        .take(max_len)
        .collect()
}
