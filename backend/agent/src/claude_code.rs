// claude_code.rs — Generic Claude Code process executor
//
// Spawns `claude --print --output-format json --system-prompt "<prompt>" "<user_prompt>"`
// and parses the JSON envelope. Concurrency is bounded by a semaphore.

use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;
use tokio::process::Command;
use tokio::sync::{Semaphore, mpsc};
use tracing::{error, info};
use uuid::Uuid;

use shared::messages::ContainerToServer;

use crate::dispatcher::DispatchMessage;

// ── Public message type ───────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ClaudeCodeMessage {
    Execute {
        session_id: Uuid,
        system_prompt: String,
        prompt: String,
    },
}

// ── Claude CLI JSON envelope ──────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct ClaudeOutput {
    result: serde_json::Value,
    is_error: bool,
}

// ── Actor ─────────────────────────────────────────────────────────────────────

pub struct ClaudeCode {
    binary: String,
    dispatch_tx: mpsc::Sender<DispatchMessage>,
    rx: mpsc::Receiver<ClaudeCodeMessage>,
    semaphore: Arc<Semaphore>,
}

impl ClaudeCode {
    pub fn new(
        binary: String,
        dispatch_tx: mpsc::Sender<DispatchMessage>,
        rx: mpsc::Receiver<ClaudeCodeMessage>,
        max_concurrent: usize,
    ) -> Self {
        Self {
            binary,
            dispatch_tx,
            rx,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn run(mut self) {
        info!("ClaudeCode actor running");
        while let Some(msg) = self.rx.recv().await {
            match msg {
                ClaudeCodeMessage::Execute {
                    session_id,
                    system_prompt,
                    prompt,
                } => {
                    let binary = self.binary.clone();
                    let sem = self.semaphore.clone();
                    let tx = self.dispatch_tx.clone();

                    tokio::spawn(async move {
                        let _permit = match sem.acquire().await {
                            Ok(p) => p,
                            Err(_) => {
                                error!("semaphore closed");
                                return;
                            }
                        };

                        info!(%session_id, "claude execution started");
                        let result = execute_claude(&binary, &system_prompt, &prompt).await;

                        let response = match result {
                            Ok(output) => {
                                if output.is_error {
                                    error!(%session_id, error = %output.result, "claude execution returned is_error");
                                    DispatchMessage::ExecutionResult(
                                        ContainerToServer::ExecutionFailed {
                                            session_id,
                                            error: output.result.to_string(),
                                        },
                                    )
                                } else {
                                    info!(%session_id, "claude execution completed successfully");
                                    DispatchMessage::ExecutionResult(
                                        ContainerToServer::ExecutionResult {
                                            session_id,
                                            output: output.result,
                                        },
                                    )
                                }
                            }
                            Err(e) => {
                                error!(%session_id, error = %e, "claude execution failed");
                                DispatchMessage::ExecutionResult(
                                    ContainerToServer::ExecutionFailed {
                                        session_id,
                                        error: e.to_string(),
                                    },
                                )
                            }
                        };

                        if tx.send(response).await.is_err() {
                            error!("dispatcher disconnected while sending execution result");
                        }
                    });
                }
            }
        }
        info!("ClaudeCode actor shutting down");
    }
}

async fn execute_claude(binary: &str, system_prompt: &str, prompt: &str) -> Result<ClaudeOutput> {
    let output = Command::new(binary)
        .arg("--print")
        .arg("--output-format")
        .arg("json")
        .arg("--system-prompt")
        .arg(system_prompt)
        .arg(prompt)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("claude exited with {}: {}", output.status, stderr.trim());
    }

    let parsed: ClaudeOutput = serde_json::from_slice(&output.stdout)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;
    use tempfile::NamedTempFile;

    /// Create a shell script that echoes a JSON response to stdout.
    fn mock_claude_script(json: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "#!/bin/sh\necho '{json}'").unwrap();
        let path = f.path().to_owned();
        std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
        f
    }

    fn mock_claude_script_fail(stderr_text: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "#!/bin/sh\necho '{stderr_text}' >&2\nexit 1").unwrap();
        let path = f.path().to_owned();
        std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
        f
    }

    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn execute_claude_success() {
        let script = mock_claude_script(r#"{"result": {"answer": 42}, "is_error": false}"#);
        let out = execute_claude(
            script.path().to_str().unwrap(),
            "you are helpful",
            "what is 6*7",
        )
        .await
        .unwrap();
        assert!(!out.is_error);
        assert_eq!(out.result["answer"], 42);
    }

    #[tokio::test]
    async fn execute_claude_is_error_flag() {
        let script = mock_claude_script(r#"{"result": "something went wrong", "is_error": true}"#);
        let out = execute_claude(script.path().to_str().unwrap(), "sys", "prompt")
            .await
            .unwrap();
        assert!(out.is_error);
    }

    #[tokio::test]
    async fn execute_claude_nonzero_exit() {
        let script = mock_claude_script_fail("bad input");
        let result = execute_claude(script.path().to_str().unwrap(), "sys", "prompt").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bad input"));
    }

    #[tokio::test]
    async fn actor_routes_success_to_dispatcher() {
        let script = mock_claude_script(r#"{"result": {"ok": true}, "is_error": false}"#);
        let binary = script.path().to_str().unwrap().to_string();

        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(16);
        let (claude_tx, claude_rx) = mpsc::channel(16);
        let actor = ClaudeCode::new(binary, dispatch_tx, claude_rx, 2);
        tokio::spawn(actor.run());

        let session_id = Uuid::now_v7();
        claude_tx
            .send(ClaudeCodeMessage::Execute {
                session_id,
                system_prompt: "sys".into(),
                prompt: "hello".into(),
            })
            .await
            .unwrap();

        let msg = tokio::time::timeout(std::time::Duration::from_secs(30), dispatch_rx.recv())
            .await
            .unwrap()
            .unwrap();

        match msg {
            DispatchMessage::ExecutionResult(ContainerToServer::ExecutionResult {
                session_id: sid,
                output,
            }) => {
                assert_eq!(sid, session_id);
                assert_eq!(output["ok"], true);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn actor_routes_is_error_as_failed() {
        let script = mock_claude_script(r#"{"result": "oops", "is_error": true}"#);
        let binary = script.path().to_str().unwrap().to_string();

        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(16);
        let (claude_tx, claude_rx) = mpsc::channel(16);
        let actor = ClaudeCode::new(binary, dispatch_tx, claude_rx, 2);
        tokio::spawn(actor.run());

        let session_id = Uuid::now_v7();
        claude_tx
            .send(ClaudeCodeMessage::Execute {
                session_id,
                system_prompt: "sys".into(),
                prompt: "hello".into(),
            })
            .await
            .unwrap();

        let msg = tokio::time::timeout(std::time::Duration::from_secs(30), dispatch_rx.recv())
            .await
            .unwrap()
            .unwrap();

        match msg {
            DispatchMessage::ExecutionResult(ContainerToServer::ExecutionFailed {
                session_id: sid,
                error,
            }) => {
                assert_eq!(sid, session_id);
                assert!(error.contains("oops"));
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn actor_routes_process_failure_as_failed() {
        let script = mock_claude_script_fail("crash");
        let binary = script.path().to_str().unwrap().to_string();

        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(16);
        let (claude_tx, claude_rx) = mpsc::channel(16);
        let actor = ClaudeCode::new(binary, dispatch_tx, claude_rx, 2);
        tokio::spawn(actor.run());

        let session_id = Uuid::now_v7();
        claude_tx
            .send(ClaudeCodeMessage::Execute {
                session_id,
                system_prompt: "sys".into(),
                prompt: "hello".into(),
            })
            .await
            .unwrap();

        let msg = tokio::time::timeout(std::time::Duration::from_secs(30), dispatch_rx.recv())
            .await
            .unwrap()
            .unwrap();

        match msg {
            DispatchMessage::ExecutionResult(ContainerToServer::ExecutionFailed {
                session_id: sid,
                ..
            }) => {
                assert_eq!(sid, session_id);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[tokio::test]
    async fn semaphore_limits_concurrency() {
        // Script that sleeps briefly to test semaphore
        let mut f = NamedTempFile::new().unwrap();
        writeln!(
            f,
            "#!/bin/sh\nsleep 0.1\necho '{{\"result\": \"done\", \"is_error\": false}}'"
        )
        .unwrap();
        let path = f.path().to_owned();
        std::fs::set_permissions(&path, PermissionsExt::from_mode(0o755)).unwrap();
        let binary = f.path().to_str().unwrap().to_string();

        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(16);
        let (claude_tx, claude_rx) = mpsc::channel(16);
        // Semaphore of 1 — only one at a time
        let actor = ClaudeCode::new(binary, dispatch_tx, claude_rx, 1);
        tokio::spawn(actor.run());

        // Send 2 tasks
        for _ in 0..2 {
            claude_tx
                .send(ClaudeCodeMessage::Execute {
                    session_id: Uuid::now_v7(),
                    system_prompt: "sys".into(),
                    prompt: "hi".into(),
                })
                .await
                .unwrap();
        }

        // Both should complete (sequentially due to semaphore=1)
        for _ in 0..2 {
            let msg = tokio::time::timeout(std::time::Duration::from_secs(30), dispatch_rx.recv())
                .await
                .unwrap()
                .unwrap();
            assert!(matches!(
                msg,
                DispatchMessage::ExecutionResult(ContainerToServer::ExecutionResult { .. })
            ));
        }
    }
}
