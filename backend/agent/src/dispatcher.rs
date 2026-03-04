use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use shared::{
    messages::{ContainerToServer, ServerToContainer},
    types::{PauseQuestion, ProposedTask, QaRoundContent, TaskContext},
};

use crate::{
    claude_code::ClaudeCodeMessage, git_manager::GitMessage, setup_ui::SetupUiMessage,
    ws_client::WsClientMessage,
};

#[derive(Debug)]
pub enum DispatchMessage {
    // From WebSocket client
    FromServer(ServerToContainer),
    // From Claude Code (Q&A generation mode)
    QuestionsGenerated {
        story_id: Uuid,
        task_id: Option<Uuid>,
        round: QaRoundContent,
    },
    TaskDecompositionReady {
        story_id: Uuid,
        tasks: Vec<ProposedTask>,
    },
    ConvergenceResult {
        story_id: Uuid,
        task_id: Option<Uuid>,
        sufficient: bool,
    },
    RefinedDescriptionReady {
        story_id: Uuid,
        stage: String,
        refined_description: String,
    },
    // From Claude Code (implementation mode)
    TaskPaused {
        task_id: Uuid,
        question: PauseQuestion,
    },
    TaskCompleted {
        task_id: Uuid,
        commit_sha: String,
    },
    TaskFailed {
        task_id: Uuid,
        error: String,
    },
    // Status
    StatusUpdate {
        task_id: Uuid,
        text: String,
    },
    // From Git Manager
    WorktreeReady {
        story_id: Uuid,
        task_id: Uuid,
        session_id: String,
        worktree: PathBuf,
        context: TaskContext,
    },
    WorktreeFailed {
        task_id: Uuid,
        error: String,
    },
    CommitComplete {
        task_id: Uuid,
        commit_sha: String,
    },
    CommitFailed {
        task_id: Uuid,
        error: String,
    },
}

pub struct Dispatcher {
    rx: mpsc::Receiver<DispatchMessage>,
    ws_tx: Option<mpsc::Sender<WsClientMessage>>,
    claude_tx: Option<mpsc::Sender<ClaudeCodeMessage>>,
    git_tx: Option<mpsc::Sender<GitMessage>>,
    _ui_tx: Option<mpsc::Sender<SetupUiMessage>>,
    /// Maps task_id → (story_id, worktree_path) for active implementation tasks.
    /// Populated when WorktreeReady arrives; cleared on completion/failure.
    task_worktrees: HashMap<Uuid, (Uuid, PathBuf)>,
}

impl Dispatcher {
    pub fn new(
        rx: mpsc::Receiver<DispatchMessage>,
        ws_tx: Option<mpsc::Sender<WsClientMessage>>,
        claude_tx: Option<mpsc::Sender<ClaudeCodeMessage>>,
        git_tx: Option<mpsc::Sender<GitMessage>>,
        ui_tx: Option<mpsc::Sender<SetupUiMessage>>,
    ) -> Self {
        Self {
            rx,
            ws_tx,
            claude_tx,
            git_tx,
            _ui_tx: ui_tx,
            task_worktrees: HashMap::new(),
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
            DispatchMessage::QuestionsGenerated {
                story_id,
                task_id,
                round,
            } => {
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::QuestionBatch {
                    story_id,
                    task_id,
                    round,
                }))
                .await;
            }
            DispatchMessage::TaskDecompositionReady { story_id, tasks } => {
                self.send_to_ws(WsClientMessage::Send(
                    ContainerToServer::TaskDecomposition {
                        story_id,
                        proposed_tasks: tasks,
                    },
                ))
                .await;
            }
            DispatchMessage::ConvergenceResult {
                story_id,
                task_id,
                sufficient,
            } => {
                // Fallback stub — only reached when refinement LLM call fails.
                info!(%story_id, ?task_id, %sufficient, "ConvergenceResult received (stub)");
            }
            DispatchMessage::RefinedDescriptionReady {
                story_id,
                stage,
                refined_description,
            } => {
                self.send_to_ws(WsClientMessage::Send(
                    ContainerToServer::RefinedDescription {
                        story_id,
                        stage,
                        refined_description,
                    },
                ))
                .await;
            }
            DispatchMessage::TaskPaused { task_id, question } => {
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::TaskPaused {
                    task_id,
                    question,
                }))
                .await;
            }
            DispatchMessage::TaskCompleted {
                task_id,
                commit_sha,
            } => {
                // If git is running, delegate commit+push to git manager (T21).
                // Git will send CommitComplete with the authoritative SHA.
                if let Some(git_tx) = &self.git_tx
                    && let Some((story_id, worktree)) = self.task_worktrees.get(&task_id)
                {
                    let msg = GitMessage::CommitAndPush {
                        story_id: *story_id,
                        task_id,
                        worktree: worktree.clone(),
                    };
                    if git_tx.send(msg).await.is_err() {
                        error!("git_manager actor disconnected during CommitAndPush");
                    }
                    return; // wait for CommitComplete / CommitFailed
                }
                // No git manager or no tracked worktree — forward directly to WS.
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::TaskCompleted {
                    task_id,
                    commit_sha,
                }))
                .await;
            }
            DispatchMessage::TaskFailed { task_id, error } => {
                self.task_worktrees.remove(&task_id);
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::TaskFailed {
                    task_id,
                    error,
                }))
                .await;
            }
            DispatchMessage::StatusUpdate { task_id, text } => {
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::StatusUpdate {
                    task_id,
                    status_text: text,
                }))
                .await;
            }
            // ── Git manager callbacks ─────────────────────────────────────────
            DispatchMessage::WorktreeReady {
                story_id,
                task_id,
                session_id,
                worktree,
                context,
            } => {
                self.task_worktrees
                    .insert(task_id, (story_id, worktree.clone()));
                self.send_to_claude(ClaudeCodeMessage::StartTask {
                    task_id,
                    session_id,
                    worktree: Some(worktree),
                    context,
                })
                .await;
            }
            DispatchMessage::WorktreeFailed { task_id, error } => {
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::TaskFailed {
                    task_id,
                    error,
                }))
                .await;
            }
            DispatchMessage::CommitComplete {
                task_id,
                commit_sha,
            } => {
                self.task_worktrees.remove(&task_id);
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::TaskCompleted {
                    task_id,
                    commit_sha,
                }))
                .await;
            }
            DispatchMessage::CommitFailed { task_id, error } => {
                self.task_worktrees.remove(&task_id);
                self.send_to_ws(WsClientMessage::Send(ContainerToServer::TaskFailed {
                    task_id,
                    error,
                }))
                .await;
            }
        }
    }

    async fn route_server_message(&mut self, msg: ServerToContainer) {
        match msg {
            ServerToContainer::StartGrooming { story_id, context } => {
                self.send_to_claude(ClaudeCodeMessage::StartGrooming { story_id, context })
                    .await;
            }
            ServerToContainer::StartPlanning { story_id, context } => {
                self.send_to_claude(ClaudeCodeMessage::StartPlanning { story_id, context })
                    .await;
            }
            ServerToContainer::AnswerReceived {
                round_id,
                answers,
                context,
            } => {
                self.send_to_claude(ClaudeCodeMessage::AnswerReceived {
                    round_id,
                    answers,
                    context,
                })
                .await;
            }
            ServerToContainer::StartTask {
                story_id,
                task_id,
                session_id,
                context,
            } => {
                // Route through git manager if available (creates branch + worktree).
                // Git will send WorktreeReady → then we send StartTask to claude.
                if let Some(git_tx) = &self.git_tx {
                    let msg = GitMessage::EnsureWorktree {
                        story_id,
                        task_id,
                        session_id,
                        context,
                    };
                    if git_tx.send(msg).await.is_err() {
                        error!("git_manager actor disconnected during EnsureWorktree");
                    }
                } else {
                    // Project mode — no git manager, run directly.
                    self.send_to_claude(ClaudeCodeMessage::StartTask {
                        task_id,
                        session_id,
                        worktree: None,
                        context,
                    })
                    .await;
                }
            }
            ServerToContainer::ResumeTask {
                task_id,
                session_id,
                answer,
            } => {
                let worktree = self.task_worktrees.get(&task_id).map(|(_, wt)| wt.clone());
                self.send_to_claude(ClaudeCodeMessage::ResumeTask {
                    task_id,
                    session_id,
                    worktree,
                    answer,
                })
                .await;
            }
            ServerToContainer::DescriptionApproved {
                story_id,
                stage,
                description,
            } => {
                self.send_to_claude(ClaudeCodeMessage::DescriptionApproved {
                    story_id,
                    stage,
                    description,
                })
                .await;
            }
            ServerToContainer::CancelTask { task_id } => {
                self.task_worktrees.remove(&task_id);
                self.send_to_claude(ClaudeCodeMessage::CancelTask { task_id })
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
    use shared::types::{GroomingContext, TaskContext};
    use uuid::Uuid;

    fn make_dispatcher(
        ws_tx: Option<mpsc::Sender<WsClientMessage>>,
        claude_tx: Option<mpsc::Sender<ClaudeCodeMessage>>,
    ) -> Dispatcher {
        let (_tx, rx) = mpsc::channel(1);
        Dispatcher::new(rx, ws_tx, claude_tx, None, None)
    }

    fn make_dispatcher_with_git(
        ws_tx: Option<mpsc::Sender<WsClientMessage>>,
        claude_tx: Option<mpsc::Sender<ClaudeCodeMessage>>,
        git_tx: Option<mpsc::Sender<GitMessage>>,
    ) -> Dispatcher {
        let (_tx, rx) = mpsc::channel(1);
        Dispatcher::new(rx, ws_tx, claude_tx, git_tx, None)
    }

    fn task_ctx() -> TaskContext {
        TaskContext {
            task_description: "do the thing".into(),
            story_decisions: vec![],
            sibling_decisions: vec![],
            knowledge: vec![],
        }
    }

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

    #[tokio::test]
    async fn questions_generated_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::QuestionsGenerated {
            story_id: Uuid::new_v4(),
            task_id: Some(Uuid::new_v4()),
            round: QaRoundContent { questions: vec![] },
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::QuestionBatch { .. })
        ));
    }

    #[tokio::test]
    async fn task_decomposition_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::TaskDecompositionReady {
            story_id: Uuid::new_v4(),
            tasks: vec![],
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::TaskDecomposition { .. })
        ));
    }

    /// TaskCompleted with no git manager → forwarded directly to WS.
    #[tokio::test]
    async fn task_completed_no_git_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::TaskCompleted {
            task_id: Uuid::new_v4(),
            commit_sha: "abc123".into(),
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::TaskCompleted { .. })
        ));
    }

    /// TaskCompleted with git manager but no tracked worktree → forwarded to WS.
    #[tokio::test]
    async fn task_completed_git_no_worktree_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let (git_tx, _git_rx) = mpsc::channel(4);
        let mut d = make_dispatcher_with_git(Some(ws_tx), None, Some(git_tx));
        d.handle(DispatchMessage::TaskCompleted {
            task_id: Uuid::new_v4(),
            commit_sha: "abc123".into(),
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected forwarded ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::TaskCompleted { .. })
        ));
    }

    /// TaskCompleted with git manager and tracked worktree → sent to git.
    #[tokio::test]
    async fn task_completed_with_worktree_routes_to_git() {
        let (git_tx, mut git_rx) = mpsc::channel(4);
        let mut d = make_dispatcher_with_git(None, None, Some(git_tx));
        let task_id = Uuid::now_v7();
        let story_id = Uuid::now_v7();
        d.task_worktrees
            .insert(task_id, (story_id, PathBuf::from("/tmp/wt")));

        d.handle(DispatchMessage::TaskCompleted {
            task_id,
            commit_sha: "irrelevant".into(),
        })
        .await;

        let msg = git_rx
            .try_recv()
            .expect("expected GitMessage::CommitAndPush");
        assert!(matches!(msg, GitMessage::CommitAndPush { task_id: tid, .. } if tid == task_id));
    }

    #[tokio::test]
    async fn task_failed_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::TaskFailed {
            task_id: Uuid::new_v4(),
            error: "something broke".into(),
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::TaskFailed { .. })
        ));
    }

    #[tokio::test]
    async fn status_update_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::StatusUpdate {
            task_id: Uuid::new_v4(),
            text: "working...".into(),
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::StatusUpdate { .. })
        ));
    }

    #[tokio::test]
    async fn start_grooming_routes_to_claude() {
        let (claude_tx, mut claude_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(None, Some(claude_tx));
        d.handle(DispatchMessage::FromServer(
            ServerToContainer::StartGrooming {
                story_id: Uuid::new_v4(),
                context: GroomingContext {
                    story_description: "test".into(),
                    knowledge: vec![],
                    codebase_context: String::new(),
                },
            },
        ))
        .await;
        let msg = claude_rx.try_recv().expect("expected a claude message");
        assert!(matches!(msg, ClaudeCodeMessage::StartGrooming { .. }));
    }

    /// StartTask with no git manager → routes directly to claude.
    #[tokio::test]
    async fn start_task_no_git_routes_to_claude() {
        let (claude_tx, mut claude_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(None, Some(claude_tx));
        d.handle(DispatchMessage::FromServer(ServerToContainer::StartTask {
            story_id: Uuid::now_v7(),
            task_id: Uuid::new_v4(),
            session_id: "sess-1".into(),
            context: task_ctx(),
        }))
        .await;
        let msg = claude_rx.try_recv().expect("expected a claude message");
        assert!(matches!(msg, ClaudeCodeMessage::StartTask { .. }));
    }

    /// StartTask with git manager → routes to git (EnsureWorktree), not directly to claude.
    #[tokio::test]
    async fn start_task_with_git_routes_to_git() {
        let (git_tx, mut git_rx) = mpsc::channel(4);
        let mut d = make_dispatcher_with_git(None, None, Some(git_tx));
        let task_id = Uuid::now_v7();
        d.handle(DispatchMessage::FromServer(ServerToContainer::StartTask {
            story_id: Uuid::now_v7(),
            task_id,
            session_id: "sess-2".into(),
            context: task_ctx(),
        }))
        .await;
        let msg = git_rx
            .try_recv()
            .expect("expected GitMessage::EnsureWorktree");
        assert!(matches!(msg, GitMessage::EnsureWorktree { task_id: tid, .. } if tid == task_id));
    }

    /// WorktreeReady → sends StartTask to claude and stores worktree mapping.
    #[tokio::test]
    async fn worktree_ready_routes_to_claude() {
        let (claude_tx, mut claude_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(None, Some(claude_tx));
        let task_id = Uuid::now_v7();
        let story_id = Uuid::now_v7();
        let wt = PathBuf::from("/tmp/wt");

        d.handle(DispatchMessage::WorktreeReady {
            story_id,
            task_id,
            session_id: "sess-3".into(),
            worktree: wt.clone(),
            context: task_ctx(),
        })
        .await;

        let msg = claude_rx.try_recv().expect("expected StartTask to claude");
        assert!(matches!(
            msg,
            ClaudeCodeMessage::StartTask { task_id: tid, worktree: Some(_), .. } if tid == task_id
        ));
        // Worktree must be tracked
        assert!(d.task_worktrees.contains_key(&task_id));
    }

    /// CommitComplete → forwards TaskCompleted to WS and removes worktree tracking.
    #[tokio::test]
    async fn commit_complete_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        let task_id = Uuid::now_v7();
        d.task_worktrees
            .insert(task_id, (Uuid::now_v7(), PathBuf::from("/tmp/wt")));

        d.handle(DispatchMessage::CommitComplete {
            task_id,
            commit_sha: "deadbeef".into(),
        })
        .await;

        let msg = ws_rx.try_recv().expect("expected TaskCompleted to WS");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::TaskCompleted { .. })
        ));
        assert!(!d.task_worktrees.contains_key(&task_id));
    }

    /// CommitFailed → forwards TaskFailed to WS.
    #[tokio::test]
    async fn commit_failed_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::CommitFailed {
            task_id: Uuid::now_v7(),
            error: "push rejected".into(),
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected TaskFailed to WS");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::TaskFailed { .. })
        ));
    }

    #[tokio::test]
    async fn cancel_task_routes_to_claude() {
        let (claude_tx, mut claude_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(None, Some(claude_tx));
        d.handle(DispatchMessage::FromServer(ServerToContainer::CancelTask {
            task_id: Uuid::new_v4(),
        }))
        .await;
        let msg = claude_rx.try_recv().expect("expected a claude message");
        assert!(matches!(msg, ClaudeCodeMessage::CancelTask { .. }));
    }

    #[tokio::test]
    async fn dispatch_convergence_result() {
        let msg = DispatchMessage::ConvergenceResult {
            story_id: Uuid::now_v7(),
            task_id: None,
            sufficient: true,
        };
        match msg {
            DispatchMessage::ConvergenceResult { sufficient, .. } => assert!(sufficient),
            _ => panic!("wrong variant"),
        }
    }

    #[tokio::test]
    async fn refined_description_ready_routes_to_ws() {
        let (ws_tx, mut ws_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(Some(ws_tx), None);
        d.handle(DispatchMessage::RefinedDescriptionReady {
            story_id: Uuid::now_v7(),
            stage: "grooming".into(),
            refined_description: "Refined text".into(),
        })
        .await;
        let msg = ws_rx.try_recv().expect("expected a ws message");
        assert!(matches!(
            msg,
            WsClientMessage::Send(ContainerToServer::RefinedDescription { .. })
        ));
    }

    #[tokio::test]
    async fn description_approved_routes_to_claude() {
        let (claude_tx, mut claude_rx) = mpsc::channel(4);
        let mut d = make_dispatcher(None, Some(claude_tx));
        d.handle(DispatchMessage::FromServer(
            ServerToContainer::DescriptionApproved {
                story_id: Uuid::now_v7(),
                stage: "planning".into(),
                description: "Approved description".into(),
            },
        ))
        .await;
        let msg = claude_rx.try_recv().expect("expected a claude message");
        assert!(matches!(msg, ClaudeCodeMessage::DescriptionApproved { .. }));
    }
}
