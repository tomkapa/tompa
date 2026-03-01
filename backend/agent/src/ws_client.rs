use std::time::Duration;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use tracing::{error, info, warn};

use shared::messages::{ContainerToServer, ServerToContainer};

use crate::{agent_status::ConnectionStatus, dispatcher::DispatchMessage};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug)]
pub enum WsClientMessage {
    Send(ContainerToServer),
}

pub struct WsClient {
    server_url: String,
    api_key: String,
    dispatch_tx: mpsc::Sender<DispatchMessage>,
    rx: mpsc::Receiver<WsClientMessage>,
    status_tx: watch::Sender<ConnectionStatus>,
}

impl WsClient {
    pub fn new(
        server_url: String,
        api_key: String,
        dispatch_tx: mpsc::Sender<DispatchMessage>,
        rx: mpsc::Receiver<WsClientMessage>,
        status_tx: watch::Sender<ConnectionStatus>,
    ) -> Self {
        Self {
            server_url,
            api_key,
            dispatch_tx,
            rx,
            status_tx,
        }
    }

    pub async fn run(mut self) {
        info!(url = %self.server_url, "ws_client starting");
        loop {
            match self.connect().await {
                Ok(ws_stream) => {
                    info!("WebSocket connected");
                    self.status_tx.send_replace(ConnectionStatus::connected());
                    self.handle_connection(ws_stream).await;
                    self.status_tx
                        .send_replace(ConnectionStatus::disconnected());
                    warn!("WebSocket disconnected, reconnecting...");
                }
                Err(e) => {
                    error!("WebSocket connection failed: {}", e);
                }
            }
            let jitter = rand::random::<u64>() % 2000;
            tokio::time::sleep(Duration::from_millis(jitter)).await;
        }
    }

    async fn connect(&self) -> Result<WsStream> {
        let url = if self.server_url.starts_with("ws://") || self.server_url.starts_with("wss://") {
            format!("{}/ws/container", self.server_url.trim_end_matches('/'))
        } else {
            format!(
                "wss://{}/ws/container",
                self.server_url.trim_end_matches('/')
            )
        };

        let mut request = url.as_str().into_client_request()?;
        request
            .headers_mut()
            .insert("Authorization", format!("Bearer {}", self.api_key).parse()?);

        let (ws_stream, _) = connect_async(request).await?;
        Ok(ws_stream)
    }

    async fn handle_connection(&mut self, ws_stream: WsStream) {
        let (mut write, mut read) = ws_stream.split();
        loop {
            tokio::select! {
                incoming = read.next() => {
                    match incoming {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<ServerToContainer>(text.as_str()) {
                                Ok(msg) => {
                                    if self.dispatch_tx
                                        .send(DispatchMessage::FromServer(msg))
                                        .await
                                        .is_err()
                                    {
                                        error!("Dispatcher closed, shutting down ws_client");
                                        return;
                                    }
                                }
                                Err(e) => warn!("Unknown server message (ignored): {e}"),
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            self.status_tx.send_modify(|s| s.record_heartbeat());
                            if write.send(Message::Pong(data)).await.is_err() {
                                return;
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("Server closed connection");
                            return;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            error!("WebSocket error: {e}");
                            return;
                        }
                        None => return,
                    }
                }

                outbound = self.rx.recv() => {
                    match outbound {
                        Some(WsClientMessage::Send(msg)) => {
                            match serde_json::to_string(&msg) {
                                Ok(text) => {
                                    if write.send(Message::Text(text.into())).await.is_err() {
                                        error!("Failed to send WebSocket message");
                                        return;
                                    }
                                }
                                Err(e) => error!("Serialization error: {e}"),
                            }
                        }
                        None => {
                            info!("ws_client channel closed");
                            return;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio::sync::mpsc;
    use tokio_tungstenite::{accept_async, tungstenite::Message as TMsg};

    use shared::messages::{ContainerToServer, ServerToContainer};

    use super::*;
    use crate::{agent_status::ConnectionStatus, dispatcher::DispatchMessage};

    async fn bind() -> (std::net::SocketAddr, TcpListener) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        (addr, listener)
    }

    fn make_client(
        addr: std::net::SocketAddr,
        dispatch_tx: mpsc::Sender<DispatchMessage>,
        rx: mpsc::Receiver<WsClientMessage>,
    ) -> WsClient {
        let (status_tx, _) = watch::channel(ConnectionStatus::default());
        WsClient::new(
            format!("ws://127.0.0.1:{}", addr.port()),
            "test-api-key".to_string(),
            dispatch_tx,
            rx,
            status_tx,
        )
    }

    #[tokio::test]
    async fn routes_server_message_to_dispatcher() {
        let (addr, listener) = bind().await;
        let (dispatch_tx, mut dispatch_rx) = mpsc::channel::<DispatchMessage>(16);
        // Keep _ws_tx alive so the channel is never closed while the test runs.
        // If it were dropped immediately, self.rx.recv() would return None and
        // win the tokio::select! before read.next() delivers the Ping.
        let (_ws_tx, ws_rx) = mpsc::channel::<WsClientMessage>(16);

        tokio::spawn(make_client(addr, dispatch_tx, ws_rx).run());

        let (tcp, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(tcp).await.unwrap();

        let json = serde_json::to_string(&ServerToContainer::Ping).unwrap();
        ws.send(TMsg::Text(json.into())).await.unwrap();

        let msg = tokio::time::timeout(Duration::from_secs(2), dispatch_rx.recv())
            .await
            .expect("timed out")
            .expect("channel closed");
        assert!(matches!(
            msg,
            DispatchMessage::FromServer(ServerToContainer::Ping)
        ));
    }

    #[tokio::test]
    async fn sends_outbound_message_to_server() {
        let (addr, listener) = bind().await;
        let (dispatch_tx, _) = mpsc::channel::<DispatchMessage>(16);
        let (ws_tx, ws_rx) = mpsc::channel::<WsClientMessage>(16);

        tokio::spawn(make_client(addr, dispatch_tx, ws_rx).run());

        let (tcp, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(tcp).await.unwrap();

        ws_tx
            .send(WsClientMessage::Send(ContainerToServer::Pong))
            .await
            .unwrap();

        let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
            .await
            .expect("timed out")
            .unwrap()
            .unwrap();

        if let TMsg::Text(text) = msg {
            let parsed: ContainerToServer = serde_json::from_str(text.as_str()).unwrap();
            assert!(matches!(parsed, ContainerToServer::Pong));
        } else {
            panic!("expected text message, got: {msg:?}");
        }
    }

    #[tokio::test]
    async fn reconnects_after_disconnect() {
        let (addr, listener) = bind().await;
        let (dispatch_tx, _) = mpsc::channel::<DispatchMessage>(16);
        let (_, ws_rx) = mpsc::channel::<WsClientMessage>(16);

        tokio::spawn(make_client(addr, dispatch_tx, ws_rx).run());

        // First connection: close immediately
        let (tcp1, _) = listener.accept().await.unwrap();
        let mut ws1 = accept_async(tcp1).await.unwrap();
        ws1.close(None).await.ok();
        drop(ws1);

        // Client should reconnect within jitter window (max 2s) + a little slack
        let (tcp2, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
            .await
            .expect("client did not reconnect in time")
            .unwrap();
        let _ws2 = accept_async(tcp2).await.unwrap();
        // reaching here confirms a second connection was established
    }
}
