use std::sync::{Arc, LazyLock};

use axum::{
    Router,
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    middleware,
    response::IntoResponse,
    routing::get,
};
use errors::UserError;
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::Mutex;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{dtos::session_dto::SessionDTO, middlewares::authentication_middleware};

pub const WEBSOCKET_ENDPOINT: &str = "/api/ws";

pub fn ws_router() -> Router {
    Router::new().nest(
        "/ws",
        Router::new()
            .route("/", get(ws_handler))
            .layer(middleware::from_fn(authentication_middleware)),
    )
}

/// Connect to the websocket.
#[utoipa::path(
    get,
    path = WEBSOCKET_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    )
)]
async fn ws_handler(ws: WebSocketUpgrade, session: SessionDTO) -> impl IntoResponse {
    return ws.on_upgrade(async move |ws_client| {
        let mut clients = CLIENTS.lock().await;
        clients.push((session.id, ws_client));
    });
}

/// A list of connected ws clients, along with their session id.
#[allow(clippy::type_complexity)]
static CLIENTS: LazyLock<Arc<Mutex<Vec<(Uuid, WebSocket)>>>> = LazyLock::new(|| Arc::new(Mutex::new(vec![])));

/// WebSocket Notifications.
#[derive(Debug, Clone, Serialize, Deserialize, Display, TS, ToSchema)]
#[ts(export)]
pub enum WsNotification {
    UpdatePosts,
    Ping { message: String, data: String },
}

impl WsNotification {
    /// Sends a notification to all connected clients.
    pub async fn send(&self) {
        let message = serde_json::to_string(self).expect("unreachable");

        let mut clients = CLIENTS.lock().await;
        let mut remove_list = vec![];

        for (client_idx, (_, client)) in clients.iter_mut().enumerate() {
            // this happens per client, so we have to clone here
            let local_message = message.clone();
            let result = client.send(Message::Text(local_message.into())).await;

            // if the message could not be sent, add the client to the remove list
            if result.is_err() {
                remove_list.push(client_idx);
            }
        }

        // remove disconnected/broken clients
        for client_idx in remove_list.into_iter().rev() {
            clients.remove(client_idx);
        }
    }
}
