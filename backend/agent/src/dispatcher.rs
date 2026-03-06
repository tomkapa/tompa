use tokio::sync::mpsc;
use tracing::{error, info, warn};

use shared::messages::{ContainerToServer, ServerToContainer};

use crate::{claude_code::ClaudeCodeMessage, ws_client::WsClientMessage};

#[derive(Debug)]
pub enum DispatchMessage {
    FromServer(ServerToContainer),
    ExecutionResult(ContainerToServer),
}

pub struct Dispatcher {
    rx: mpsc::Receiver<DispatchMessage>,
    ws_tx: Option<mpsc::Sender<WsClientMessage>>,
    claude_tx: Option<mpsc::Sender<ClaudeCodeMessage>>,
}

impl Dispatcher {
    pub fn new(
        rx: mpsc::Receiver<DispatchMessage>,
        ws_tx: Option<mpsc::Sender<WsClientMessage>>,
        claude_tx: Option<mpsc::Sender<ClaudeCodeMessage>>,
    ) -> Self {
        Self {
            rx,
            ws_tx,
            claude_tx,
        }
    }

    pub async fn run(mut self) {
        info!("Dispatcher running");
        while let Some(msg) = self.rx.recv().await {
            self.handle(msg).await;
        }
        info!("Dispatcher channel closed, shutting down");
    }

    pub async fn handle(&mut self, msg: DispatchMessage) {
        match msg {
            DispatchMessage::FromServer(server_msg) => {
                self.route_server_message(server_msg).await;
            }
            DispatchMessage::ExecutionResult(result) => {
                self.send_to_ws(WsClientMessage::Send(result)).await;
            }
        }
    }

    async fn route_server_message(&mut self, msg: ServerToContainer) {
        match msg {
            ServerToContainer::Execute {
                session_id,
                system_prompt,
                prompt,
            } => {
                self.send_to_claude(ClaudeCodeMessage::Execute {
                    session_id,
                    system_prompt,
                    prompt,
                })
                .await;
            }
            ServerToContainer::Ping => {
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::Pong))
                    .await;
            }
        }
    }

    async fn send_to_ws(&self, msg: WsClientMessage) {
        if let Some(tx) = &self.ws_tx {
            if tx.send(msg).await.is_err() {
                error!("ws_client actor disconnected");
            }
        } else {
            warn!("ws_client not running, dropping outbound message");
        }
    }

    async fn send_to_claude(&self, msg: ClaudeCodeMessage) {
        if let Some(tx) = &self.claude_tx {
            if tx.send(msg).await.is_err() {
                error!("claude_code actor disconnected");
            }
        } else {
            warn!("claude_code not running, dropping message");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_dispatcher(
        ws_tx: Option<mpsc::Sender<WsClientMessage>>,
        claude_tx: Option<mpsc::Sender<ClaudeCodeMessage>>,
    ) -> Dispatcher {
        let (_tx, rx) = mpsc::channel(1);
        Dispatcher::new(rx, ws_tx, claude_tx)
    }

    // ── Ping/Pong ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn ping_sends_pong_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::FromServer(ServerToContainer::Ping))
            .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::Pong)
        ));
    }

    // ── Execute routing ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn execute_routes_to_claude() {
        let (claude_tx, mut claude_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(None, Some(claude_tx));
        let session_id = Uuid::now_v7();
        d.handle(DispatchMessage::FromServer(ServerToContainer::Execute {
            session_id,
            system_prompt: "you are helpful".into(),
            prompt: "hello".into(),
        }))
        .await;
        let msg = claude_rx.try_recv().expect("expected a claude message");
        match msg {
            ClaudeCodeMessage::Execute {
                session_id: sid,
                system_prompt,
                prompt,
            } => {
                assert_eq!(sid, session_id);
                assert_eq!(system_prompt, "you are helpful");
                assert_eq!(prompt, "hello");
            }
        }
    }

    // ── ExecutionResult routing ──────────────────────────────────────────────

    #[tokio::test]
    async fn execution_result_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        let session_id = Uuid::now_v7();
        d.handle(DispatchMessage::ExecutionResult(
            ContainerToServer::ExecutionResult {
                session_id,
                output: serde_json::json!({"answer": 42}),
            },
        ))
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        match msg {
            WsClientMessage::Send(ContainerToServer::ExecutionResult {
                session_id: sid,
                output,
            }) => {
                assert_eq!(sid, session_id);
                assert_eq!(output["answer"], 42);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn execution_failed_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        let session_id = Uuid::now_v7();
        d.handle(DispatchMessage::ExecutionResult(
            ContainerToServer::ExecutionFailed {
                session_id,
                error: "something broke".into(),
            },
        ))
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        match msg {
            WsClientMessage::Send(ContainerToServer::ExecutionFailed {
                session_id: sid,
                error,
            }) => {
                assert_eq!(sid, session_id);
                assert_eq!(error, "something broke");
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    // ── No actor available ───────────────────────────────────────────────────

    #[tokio::test]
    async fn execute_no_claude_does_not_panic() {
        let mut d = make_dispatcher(None, None);
        d.handle(DispatchMessage::FromServer(ServerToContainer::Execute {
            session_id: Uuid::now_v7(),
            system_prompt: "sys".into(),
            prompt: "hello".into(),
        }))
        .await;
        // Should not panic — just logs a warning
    }

    #[tokio::test]
    async fn ping_no_ws_does_not_panic() {
        let mut d = make_dispatcher(None, None);
        d.handle(DispatchMessage::FromServer(ServerToContainer::Ping))
            .await;
        // Should not panic — just logs a warning
    }

    #[tokio::test]
    async fn execution_result_no_ws_does_not_panic() {
        let mut d = make_dispatcher(None, None);
        d.handle(DispatchMessage::ExecutionResult(
            ContainerToServer::ExecutionResult {
                session_id: Uuid::now_v7(),
                output: serde_json::json!(null),
            },
        ))
        .await;
        // Should not panic
    }

    // ── Disconnected actors ──────────────────────────────────────────────────

    #[tokio::test]
    async fn execute_with_dropped_claude_rx_does_not_panic() {
        let (claude_tx, claude_rx) = mpsc::channel(4);
        drop(claude_rx); // simulate disconnected actor
        let mut d = make_dispatcher(None, Some(claude_tx));
        d.handle(DispatchMessage::FromServer(ServerToContainer::Execute {
            session_id: Uuid::now_v7(),
            system_prompt: "sys".into(),
            prompt: "hello".into(),
        }))
        .await;
        // Should not panic — logs error about disconnected actor
    }

    #[tokio::test]
    async fn pong_with_dropped_ws_rx_does_not_panic() {
        let (ws_tx, ws_rx) = mpsc::channel(4);
        drop(ws_rx);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::FromServer(ServerToContainer::Ping))
            .await;
        // Should not panic
    }
}
