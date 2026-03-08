use std::sync::{Arc, LazyLock};

use axum::{
    Router,
    extract::{
        WebSocketUpgrade,
        ws::Message,
    },
    middleware,
    response::IntoResponse,
    routing::get,
};
use errors::UserError;
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::{Mutex, mpsc};
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
///
/// Auth: Required
/// Permissions: None
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
    return ws.on_upgrade(async move |mut ws_client| {
        let id = session.id;
        let (tx, mut rx) = mpsc::channel::<String>(32);

        {
            let mut clients = CLIENTS.lock().await;
            clients.push((id, tx));
        }

        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Some(text) => {
                            if ws_client.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                msg = ws_client.recv() => {
                    match msg {
                        Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                        _ => {}
                    }
                }
            }
        }

        let mut clients = CLIENTS.lock().await;
        clients.retain(|(client_id, _)| *client_id != id);
    });
}

static CLIENTS: LazyLock<Arc<Mutex<Vec<(Uuid, mpsc::Sender<String>)>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(vec![])));

/// WebSocket Notifications.
#[derive(Debug, Clone, Serialize, Deserialize, Display, TS, ToSchema)]
#[ts(export)]
pub enum WsNotification {
    UpdatePosts,
    Ping { message: String, data: String },
}

impl WsNotification {
    pub async fn send(&self) {
        let message = serde_json::to_string(self).expect("unreachable");

        let mut clients = CLIENTS.lock().await;
        let mut remove_list = vec![];

        for (client_idx, (_, tx)) in clients.iter().enumerate() {
            if tx.send(message.clone()).await.is_err() {
                remove_list.push(client_idx);
            }
        }

        // remove disconnected clients
        for client_idx in remove_list.into_iter().rev() {
            clients.remove(client_idx);
        }
    }
}
