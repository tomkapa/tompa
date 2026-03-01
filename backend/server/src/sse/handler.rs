use std::{
    convert::Infallible,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    extract::{Extension, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::get,
    Router,
};
use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::{
    auth::{middleware::require_auth, types::AuthContext},
    state::AppState,
};

use super::broadcaster::{SseBroadcaster, SseEvent};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/events/stream", get(sse_handler))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// Calls `SseBroadcaster::unsubscribe` when dropped, ensuring the client
/// is removed from the broadcaster even if no further events are sent.
struct DropGuard {
    broadcaster: Arc<SseBroadcaster>,
    org_id: Uuid,
    sender_id: Uuid,
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        self.broadcaster.unsubscribe(self.org_id, self.sender_id);
    }
}

/// Wraps an inner SSE stream and keeps the `DropGuard` alive for the
/// lifetime of the stream so that the client is cleaned up on disconnect.
struct SseStream {
    inner: futures_util::stream::BoxStream<'static, Result<Event, Infallible>>,
    _guard: DropGuard,
}

impl Stream for SseStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // BoxStream is Unpin (it's Pin<Box<dyn Stream>>; Box<T>: Unpin).
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }
}

/// `GET /api/v1/events/stream`
///
/// Establishes a persistent SSE connection. All `SseEvent`s broadcast for
/// the authenticated user's `org_id` are forwarded to this client.
/// The connection is cleaned up automatically when the client disconnects.
async fn sse_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    let (sender_id, rx) = state.broadcaster.subscribe(auth.org_id);

    let guard = DropGuard {
        broadcaster: Arc::clone(&state.broadcaster),
        org_id: auth.org_id,
        sender_id,
    };

    let inner = UnboundedReceiverStream::new(rx)
        .map(|event: SseEvent| {
            let name = event.event_name();
            let data = serde_json::to_string(&event).unwrap_or_default();
            Ok::<Event, Infallible>(Event::default().event(name).data(data))
        })
        .boxed();

    Sse::new(SseStream { inner, _guard: guard }).keep_alive(KeepAlive::default())
}
