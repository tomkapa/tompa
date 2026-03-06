use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    extract::{
        FromRequestParts, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use shared::messages::{ContainerToServer, ServerToContainer};
use tokio::sync::mpsc;

use crate::{
    agents::service,
    auth::types::AuthError,
    container_keys::{repo as key_repo, service::verify_api_key, types::ContainerKeyInfo},
    state::AppState,
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const MAX_MISSED_PONGS: u8 = 2;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/ws/container", get(ws_handler))
        .with_state(state)
}

/// `GET /ws/container`
///
/// Performs Bearer-token authentication before accepting the WebSocket
/// upgrade.  Returns 401 if the token is absent or invalid.
///
/// Auth is checked manually before extracting `WebSocketUpgrade` so that
/// missing/invalid tokens always produce 401 — even when the request lacks
/// the `OnUpgrade` extension (e.g. in `tower::ServiceExt::oneshot` tests).
async fn ws_handler(State(state): State<AppState>, req: axum::extract::Request) -> Response {
    let (mut parts, _body) = req.into_parts();

    // 1. Authenticate via Bearer token.
    let raw_key = match extract_bearer(&parts) {
        Some(k) => k,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let key_info = match verify_api_key(&state.pool, &raw_key).await {
        Ok(info) => info,
        Err(AuthError::InvalidToken) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // 2. Attempt WebSocket upgrade (requires the OnUpgrade extension).
    match WebSocketUpgrade::from_request_parts(&mut parts, &state).await {
        Ok(ws) => ws
            .on_upgrade(move |socket| handle_socket(socket, state, key_info))
            .into_response(),
        Err(rejection) => rejection.into_response(),
    }
}

fn extract_bearer(parts: &Parts) -> Option<String> {
    let value = parts.headers.get(header::AUTHORIZATION)?;
    let s = value.to_str().ok()?;
    s.strip_prefix("Bearer ").map(str::to_string)
}

async fn handle_socket(socket: WebSocket, state: AppState, key_info: ContainerKeyInfo) {
    let key_id = key_info.key_id;

    tracing::info!(%key_id, org_id = %key_info.org_id, "container agent connected");

    // Stamp last_connected_at (best-effort — do not abort on DB failure).
    let _ = key_repo::update_last_connected(&state.pool, key_id).await;

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<ServerToContainer>();

    // The sender lives in the registry; external callers use `registry.send_to`
    // to enqueue outbound messages.
    state.registry.register(key_id, msg_tx);

    // Write task — drains the outbound channel, serialises each message, and
    // forwards it to the WebSocket sink.
    let write_handle = tokio::spawn(async move {
        while let Some(msg) = msg_rx.recv().await {
            let text = match serde_json::to_string(&msg) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if ws_tx.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    // Read + heartbeat loop runs on the current task so we can return a
    // single cleanup point.
    let missed_pongs = Arc::new(AtomicU8::new(0));
    let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
    // Skip the immediate first tick so the first ping fires after the full
    // interval, not instantly on connect.
    interval.tick().await;

    loop {
        tokio::select! {
            biased;

            frame = ws_rx.next() => {
                match frame {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ContainerToServer>(&text) {
                            Ok(ContainerToServer::Pong) => {
                                missed_pongs.store(0, Ordering::Relaxed);
                            }
                            Ok(msg) => dispatch(&state, &key_info, msg).await,
                            Err(_) => {} // ignore malformed frames
                        }
                    }
                    // Graceful close, stream end, or transport error.
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {} // Binary / Ping / Pong frames — not expected, ignore
                }
            }

            _ = interval.tick() => {
                if missed_pongs.load(Ordering::Relaxed) >= MAX_MISSED_PONGS {
                    // Two consecutive pings without a pong → treat as dead.
                    break;
                }
                missed_pongs.fetch_add(1, Ordering::Relaxed);
                // Ignore send errors; the write task will fail naturally once
                // the channel is closed after `unregister`.
                let _ = state.registry.send_to(key_id, ServerToContainer::Ping);
            }
        }
    }

    // Cancel the write task and remove the connection from the registry.
    // Dropping the registry entry closes the channel, which causes msg_rx
    // inside the write task to return None — but we abort proactively to
    // avoid any delay.
    write_handle.abort();
    state.registry.unregister(key_id);

    tracing::info!(%key_id, org_id = %key_info.org_id, "container agent disconnected");
}

/// Route an authenticated `ContainerToServer` message to the service layer.
async fn dispatch(state: &AppState, key_info: &ContainerKeyInfo, msg: ContainerToServer) {
    service::handle_message(state, key_info, msg).await;
}
