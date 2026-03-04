// claude_code.rs — Claude Code Supervisor
//
// All LLM interaction goes through the Claude Code CLI subprocess:
//   1. Q&A Generation Mode  — one-shot `claude --print --output-format json` calls
//   2. Implementation Mode  — session-based subprocess with pause/resume via stdout markers
//
// The binary path defaults to "claude"; set CLAUDE_CMD env var to override (useful in tests).

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{Result, anyhow};
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use shared::enums::TaskType;
use shared::types::{
    Answer, AnswerContext, GroomingContext, PauseQuestion, PlanningContext, ProposedTask,
    QaDecision, QaRoundContent, Question, TaskContext,
};

use crate::dispatcher::DispatchMessage;
use crate::prompts;

// ── Public message type ───────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ClaudeCodeMessage {
    // Q&A generation mode
    StartGrooming {
        story_id: Uuid,
        context: GroomingContext,
    },
    StartPlanning {
        story_id: Uuid,
        context: PlanningContext,
    },
    AnswerReceived {
        round_id: Uuid,
        answers: Vec<Answer>,
        context: AnswerContext,
    },
    // Description approval after refinement
    DescriptionApproved {
        story_id: Uuid,
        stage: String,
        description: String,
    },
    // Implementation mode
    StartTask {
        task_id: Uuid,
        session_id: String,
        worktree: Option<PathBuf>,
        context: TaskContext,
    },
    ResumeTask {
        task_id: Uuid,
        session_id: String,
        worktree: Option<PathBuf>,
        answer: Answer,
    },
    CancelTask {
        task_id: Uuid,
    },
}

// ── Internal types ────────────────────────────────────────────────────────────

/// Active Q&A session — at most one at a time.
enum QaSession {
    Grooming {
        story_id: Uuid,
        context: GroomingContext,
        decisions: Vec<QaDecision>,
        last_questions: Vec<Question>,
    },
    Planning {
        story_id: Uuid,
        context: PlanningContext,
        decisions: Vec<QaDecision>,
        last_questions: Vec<Question>,
    },
}

/// Results forwarded from implementation monitoring tasks back to the actor.
#[derive(Debug)]
enum ImplResult {
    Paused {
        task_id: Uuid,
        question: PauseQuestion,
    },
    Completed {
        task_id: Uuid,
        commit_sha: String,
    },
    Failed {
        task_id: Uuid,
        error: String,
    },
}

/// JSON envelope emitted by `claude --output-format json`.
#[derive(Deserialize)]
struct ClaudeOutput {
    result: String,
    #[serde(default)]
    is_error: bool,
}

/// Raw option shape as the model outputs it.
#[derive(Deserialize)]
struct RawQuestionOption {
    label: String,
    pros: String,
    cons: String,
}

/// Raw question shape as the model outputs it (no id — we assign UUIDs).
#[derive(Deserialize)]
struct RawQuestion {
    text: String,
    domain: String,
    rationale: String,
    options: Vec<RawQuestionOption>,
    recommended_option_index: usize,
}

#[derive(Deserialize)]
struct RawQaRound {
    questions: Vec<RawQuestion>,
}

/// Raw proposed-task shape.
#[derive(Deserialize)]
struct RawTask {
    name: String,
    description: String,
    task_type: String,
    position: i32,
    #[serde(default)]
    depends_on: Vec<i32>,
}

#[derive(Deserialize)]
struct RawDecomposition {
    tasks: Vec<RawTask>,
}

/// Preserved context for task decomposition after planning description approval.
struct PendingDecomposition {
    story_id: Uuid,
    context: PlanningContext,
    decisions: Vec<QaDecision>,
}

// ── Actor ─────────────────────────────────────────────────────────────────────

pub struct ClaudeCode {
    /// Path to the `claude` binary (overridable via CLAUDE_CMD).
    binary: String,
    dispatch_tx: mpsc::Sender<DispatchMessage>,
    rx: mpsc::Receiver<ClaudeCodeMessage>,
    /// Channel from monitoring tasks back to this actor.
    monitor_tx: mpsc::Sender<ImplResult>,
    monitor_rx: mpsc::Receiver<ImplResult>,
    /// At most one active Q&A session (grooming or planning).
    active_qa: Option<QaSession>,
    /// Active implementation subprocesses keyed by task_id.
    active_impl: HashMap<Uuid, Child>,
    /// Stashed context for task decomposition after planning description approval.
    pending_planning_decomposition: Option<PendingDecomposition>,
}

impl ClaudeCode {
    pub fn new(
        binary: String,
        dispatch_tx: mpsc::Sender<DispatchMessage>,
        rx: mpsc::Receiver<ClaudeCodeMessage>,
    ) -> Self {
        let (monitor_tx, monitor_rx) = mpsc::channel(64);
        Self {
            binary,
            dispatch_tx,
            rx,
            monitor_tx,
            monitor_rx,
            active_qa: None,
            active_impl: HashMap::new(),
            pending_planning_decomposition: None,
        }
    }

    pub async fn run(mut self) {
        info!("claude_code actor starting");
        loop {
            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(m) => self.handle_message(m).await,
                        None => break,
                    }
                }
                Some(result) = self.monitor_rx.recv() => {
                    self.handle_impl_result(result).await;
                }
            }
        }
        // Kill any lingering subprocesses on shutdown.
        for (task_id, mut child) in self.active_impl {
            info!(%task_id, "killing active implementation on shutdown");
            let _ = child.kill().await;
        }
        info!("claude_code actor shut down");
    }

    async fn handle_message(&mut self, msg: ClaudeCodeMessage) {
        match msg {
            ClaudeCodeMessage::StartGrooming { story_id, context } => {
                self.handle_start_grooming(story_id, context).await;
            }
            ClaudeCodeMessage::StartPlanning { story_id, context } => {
                self.handle_start_planning(story_id, context).await;
            }
            ClaudeCodeMessage::AnswerReceived {
                round_id,
                answers,
                context,
            } => {
                self.handle_answer_received(round_id, answers, context)
                    .await;
            }
            ClaudeCodeMessage::DescriptionApproved {
                story_id,
                stage,
                description,
            } => {
                self.handle_description_approved(story_id, &stage, &description)
                    .await;
            }
            ClaudeCodeMessage::StartTask {
                task_id,
                session_id,
                worktree,
                context,
            } => {
                self.handle_start_task(task_id, session_id, worktree, context)
                    .await;
            }
            ClaudeCodeMessage::ResumeTask {
                task_id,
                session_id,
                worktree,
                answer,
            } => {
                self.handle_resume_task(task_id, session_id, worktree, answer)
                    .await;
            }
            ClaudeCodeMessage::CancelTask { task_id } => {
                self.handle_cancel_task(task_id).await;
            }
        }
    }

    // ── Q&A generation ────────────────────────────────────────────────────────

    async fn handle_start_grooming(&mut self, story_id: Uuid, context: GroomingContext) {
        info!(%story_id, "generating grooming questions");
        let questions = self.generate_grooming_questions(&context, &[]).await;
        if questions.is_empty() {
            error!(%story_id, "failed to generate any grooming questions");
            return;
        }
        let round = QaRoundContent {
            questions: questions.clone(),
        };
        self.active_qa = Some(QaSession::Grooming {
            story_id,
            context,
            decisions: vec![],
            last_questions: questions,
        });
        self.send(DispatchMessage::QuestionsGenerated {
            story_id,
            task_id: None,
            round,
        })
        .await;
    }

    async fn handle_start_planning(&mut self, story_id: Uuid, context: PlanningContext) {
        info!(%story_id, "generating planning questions");
        let prompt = prompts::planning::build_planning_prompt(&context, &[]);
        match self.generate_qa(&prompt).await {
            Ok(round) => {
                let questions = round.questions.clone();
                self.active_qa = Some(QaSession::Planning {
                    story_id,
                    context,
                    decisions: vec![],
                    last_questions: questions,
                });
                self.send(DispatchMessage::QuestionsGenerated {
                    story_id,
                    task_id: None,
                    round,
                })
                .await;
            }
            Err(e) => error!(%story_id, %e, "planning question generation failed"),
        }
    }

    async fn handle_answer_received(
        &mut self,
        round_id: Uuid,
        answers: Vec<Answer>,
        recovery: AnswerContext,
    ) {
        // Fast path: use in-memory session if available.
        if let Some(qa) = self.active_qa.take() {
            match qa {
                QaSession::Grooming {
                    story_id,
                    context,
                    decisions,
                    last_questions,
                } => {
                    self.process_grooming_answers(
                        story_id,
                        context,
                        decisions,
                        last_questions,
                        answers,
                    )
                    .await;
                }
                QaSession::Planning {
                    story_id,
                    context,
                    decisions,
                    last_questions,
                } => {
                    self.process_planning_answers(
                        story_id,
                        context,
                        decisions,
                        last_questions,
                        answers,
                    )
                    .await;
                }
            }
            return;
        }

        // Cold path: reconstruct session from the recovery context.
        info!(%round_id, stage = %recovery.stage, "recovering QA session from AnswerReceived context");
        match recovery.stage.as_str() {
            "grooming" => {
                let Some(context) = recovery.grooming_context else {
                    error!(%round_id, "recovery context missing grooming_context");
                    return;
                };
                self.process_grooming_answers(
                    recovery.story_id,
                    context,
                    recovery.prior_decisions,
                    recovery.questions,
                    answers,
                )
                .await;
            }
            "planning" => {
                let Some(context) = recovery.planning_context else {
                    error!(%round_id, "recovery context missing planning_context");
                    return;
                };
                self.process_planning_answers(
                    recovery.story_id,
                    context,
                    recovery.prior_decisions,
                    recovery.questions,
                    answers,
                )
                .await;
            }
            other => {
                warn!(%round_id, %other, "AnswerReceived for unhandled stage — ignoring");
            }
        }
    }

    async fn process_grooming_answers(
        &mut self,
        story_id: Uuid,
        context: GroomingContext,
        mut decisions: Vec<QaDecision>,
        last_questions: Vec<Question>,
        answers: Vec<Answer>,
    ) {
        append_decisions(&mut decisions, &last_questions, &answers);

        let convergence_prompt =
            prompts::convergence::build_convergence_prompt(&context.story_description, &decisions);
        let sufficient = self
            .assess_convergence(&convergence_prompt)
            .await
            .unwrap_or(false);

        if sufficient {
            info!(%story_id, "grooming convergence: SUFFICIENT — generating refined description");
            let refinement_prompt = prompts::description_refinement::build_refinement_prompt(
                &context.story_description,
                &decisions,
                "grooming",
            );
            match self.generate_refined_description(&refinement_prompt).await {
                Ok(refined) => {
                    self.send(DispatchMessage::RefinedDescriptionReady {
                        story_id,
                        stage: "grooming".into(),
                        refined_description: refined,
                    })
                    .await;
                }
                Err(e) => {
                    error!(%story_id, %e, "refinement LLM call failed — falling back to convergence stub");
                    self.send(DispatchMessage::ConvergenceResult {
                        story_id,
                        task_id: None,
                        sufficient: true,
                    })
                    .await;
                }
            }
        } else {
            info!(%story_id, "grooming convergence: CONTINUE");
            let questions = self.generate_grooming_questions(&context, &decisions).await;
            if !questions.is_empty() {
                let round = QaRoundContent {
                    questions: questions.clone(),
                };
                self.active_qa = Some(QaSession::Grooming {
                    story_id,
                    context,
                    decisions,
                    last_questions: questions,
                });
                self.send(DispatchMessage::QuestionsGenerated {
                    story_id,
                    task_id: None,
                    round,
                })
                .await;
            }
        }
    }

    async fn process_planning_answers(
        &mut self,
        story_id: Uuid,
        context: PlanningContext,
        mut decisions: Vec<QaDecision>,
        last_questions: Vec<Question>,
        answers: Vec<Answer>,
    ) {
        append_decisions(&mut decisions, &last_questions, &answers);

        let convergence_prompt =
            prompts::convergence::build_convergence_prompt(&context.story_description, &decisions);
        let sufficient = self
            .assess_convergence(&convergence_prompt)
            .await
            .unwrap_or(false);

        if sufficient {
            info!(%story_id, "planning convergence: SUFFICIENT — generating refined description");
            let refinement_prompt = prompts::description_refinement::build_refinement_prompt(
                &context.story_description,
                &decisions,
                "planning",
            );
            match self.generate_refined_description(&refinement_prompt).await {
                Ok(refined) => {
                    self.pending_planning_decomposition = Some(PendingDecomposition {
                        story_id,
                        context,
                        decisions,
                    });
                    self.send(DispatchMessage::RefinedDescriptionReady {
                        story_id,
                        stage: "planning".into(),
                        refined_description: refined,
                    })
                    .await;
                }
                Err(e) => {
                    // Fallback: run decomposition inline (old behavior).
                    error!(%story_id, %e, "refinement LLM call failed — falling back to inline decomposition");
                    let decomp_prompt = prompts::task_decomposition::build_decomposition_prompt(
                        &context, &decisions,
                    );
                    match self.generate_decomposition(&decomp_prompt).await {
                        Ok(tasks) => {
                            self.send(DispatchMessage::TaskDecompositionReady { story_id, tasks })
                                .await;
                        }
                        Err(e2) => {
                            error!(%story_id, %e2, "task decomposition fallback also failed")
                        }
                    }
                }
            }
        } else {
            info!(%story_id, "planning convergence: CONTINUE");
            let prompt = prompts::planning::build_planning_prompt(&context, &decisions);
            match self.generate_qa(&prompt).await {
                Ok(round) => {
                    let questions = round.questions.clone();
                    self.active_qa = Some(QaSession::Planning {
                        story_id,
                        context,
                        decisions,
                        last_questions: questions,
                    });
                    self.send(DispatchMessage::QuestionsGenerated {
                        story_id,
                        task_id: None,
                        round,
                    })
                    .await;
                }
                Err(e) => error!(%story_id, %e, "follow-up planning questions failed"),
            }
        }
    }

    // ── Description refinement ─────────────────────────────────────────────────

    /// One-shot call that returns the refined description as plain text.
    async fn generate_refined_description(&self, prompt: &str) -> Result<String> {
        let output = Command::new(&self.binary)
            .arg("--print")
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| anyhow!("failed to spawn {}: {}", self.binary, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("claude exited non-zero: {stderr}"));
        }

        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout.trim().to_string())
    }

    async fn handle_description_approved(
        &mut self,
        story_id: Uuid,
        stage: &str,
        description: &str,
    ) {
        match stage {
            "planning" => {
                let pending = match self.pending_planning_decomposition.take() {
                    Some(p) => p,
                    None => {
                        error!(%story_id, "DescriptionApproved for planning but no pending decomposition — agent may have restarted");
                        return;
                    }
                };
                debug_assert_eq!(
                    pending.story_id, story_id,
                    "story_id mismatch in PendingDecomposition"
                );
                let mut context = pending.context;
                context.story_description = description.to_string();
                let decomp_prompt = prompts::task_decomposition::build_decomposition_prompt(
                    &context,
                    &pending.decisions,
                );
                match self.generate_decomposition(&decomp_prompt).await {
                    Ok(tasks) => {
                        self.send(DispatchMessage::TaskDecompositionReady { story_id, tasks })
                            .await;
                    }
                    Err(e) => error!(%story_id, %e, "task decomposition after approval failed"),
                }
            }
            "grooming" => {
                // No-op on the agent side — server handles starting planning.
                info!(%story_id, "grooming description approved — server will start planning");
            }
            other => {
                warn!(%story_id, %other, "DescriptionApproved for unhandled stage");
            }
        }
    }

    // ── Implementation mode ───────────────────────────────────────────────────

    async fn handle_start_task(
        &mut self,
        task_id: Uuid,
        session_id: String,
        worktree: Option<PathBuf>,
        context: TaskContext,
    ) {
        info!(%task_id, %session_id, ?worktree, "starting implementation");
        let prompt = prompts::implementation::build_implementation_prompt(&context);
        match self
            .spawn_impl(task_id, &session_id, &prompt, false, worktree.as_deref())
            .await
        {
            Ok(child) => {
                self.active_impl.insert(task_id, child);
            }
            Err(e) => {
                error!(%task_id, %e, "failed to start implementation");
                self.send(DispatchMessage::TaskFailed {
                    task_id,
                    error: e.to_string(),
                })
                .await;
            }
        }
    }

    async fn handle_resume_task(
        &mut self,
        task_id: Uuid,
        session_id: String,
        worktree: Option<PathBuf>,
        answer: Answer,
    ) {
        info!(%task_id, %session_id, "resuming implementation");
        // Previous process already exited at the decision point; clean up if still present.
        let _ = self.active_impl.remove(&task_id);
        match self
            .spawn_impl(
                task_id,
                &session_id,
                &answer.selected_answer_text,
                true,
                worktree.as_deref(),
            )
            .await
        {
            Ok(child) => {
                self.active_impl.insert(task_id, child);
            }
            Err(e) => {
                error!(%task_id, %e, "failed to resume implementation");
                self.send(DispatchMessage::TaskFailed {
                    task_id,
                    error: e.to_string(),
                })
                .await;
            }
        }
    }

    async fn handle_cancel_task(&mut self, task_id: Uuid) {
        if let Some(mut child) = self.active_impl.remove(&task_id) {
            info!(%task_id, "cancelling implementation subprocess");
            if let Err(e) = child.kill().await {
                warn!(%task_id, %e, "kill failed");
            }
        } else {
            warn!(%task_id, "CancelTask but no active subprocess");
        }
    }

    async fn handle_impl_result(&mut self, result: ImplResult) {
        match result {
            ImplResult::Paused { task_id, question } => {
                self.active_impl.remove(&task_id);
                self.send(DispatchMessage::TaskPaused { task_id, question })
                    .await;
            }
            ImplResult::Completed {
                task_id,
                commit_sha,
            } => {
                self.active_impl.remove(&task_id);
                self.send(DispatchMessage::TaskCompleted {
                    task_id,
                    commit_sha,
                })
                .await;
            }
            ImplResult::Failed { task_id, error } => {
                self.active_impl.remove(&task_id);
                self.send(DispatchMessage::TaskFailed { task_id, error })
                    .await;
            }
        }
    }

    // ── Core subprocess helpers ───────────────────────────────────────────────

    /// One-shot Claude Code call that returns a structured QA round.
    async fn generate_qa(&self, prompt: &str) -> Result<QaRoundContent> {
        let output = Command::new(&self.binary)
            .arg("--print")
            .arg("--output-format")
            .arg("json")
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| anyhow!("failed to spawn {}: {}", self.binary, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("claude exited non-zero: {stderr}"));
        }

        let stdout = String::from_utf8(output.stdout)?;
        let envelope: ClaudeOutput = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("bad claude envelope: {e} — raw: {stdout:.200}"))?;

        if envelope.is_error {
            return Err(anyhow!("claude error: {}", envelope.result));
        }

        let json = strip_markdown_fences(&envelope.result);
        let raw: RawQaRound = serde_json::from_str(json)
            .map_err(|e| anyhow!("bad QA JSON: {} — result: {:.200}", e, envelope.result))?;

        let questions = raw
            .questions
            .into_iter()
            .map(|q| Question {
                id: Uuid::now_v7(),
                text: q.text,
                domain: q.domain,
                rationale: q.rationale,
                options: q
                    .options
                    .into_iter()
                    .map(|o| shared::types::QuestionOption {
                        label: o.label,
                        pros: o.pros,
                        cons: o.cons,
                    })
                    .collect(),
                recommended_option_index: q.recommended_option_index,
            })
            .collect();

        Ok(QaRoundContent { questions })
    }

    /// One-shot convergence check. Returns `true` if the model says `SUFFICIENT`.
    async fn assess_convergence(&self, prompt: &str) -> Result<bool> {
        let output = Command::new(&self.binary)
            .arg("--print")
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| anyhow!("failed to spawn {}: {}", self.binary, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("claude exited non-zero: {stderr}"));
        }

        let stdout = String::from_utf8(output.stdout)?;
        Ok(parse_convergence_response(&stdout))
    }

    /// One-shot decomposition call. Returns the proposed task list.
    async fn generate_decomposition(&self, prompt: &str) -> Result<Vec<ProposedTask>> {
        let output = Command::new(&self.binary)
            .arg("--print")
            .arg("--output-format")
            .arg("json")
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| anyhow!("failed to spawn {}: {}", self.binary, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("claude exited non-zero: {stderr}"));
        }

        let stdout = String::from_utf8(output.stdout)?;
        let envelope: ClaudeOutput =
            serde_json::from_str(&stdout).map_err(|e| anyhow!("bad claude envelope: {e}"))?;

        if envelope.is_error {
            return Err(anyhow!("claude error: {}", envelope.result));
        }

        let json = strip_markdown_fences(&envelope.result);
        let raw: RawDecomposition =
            serde_json::from_str(json).map_err(|e| anyhow!("bad decomposition JSON: {e}"))?;

        let tasks = raw
            .tasks
            .into_iter()
            .map(|t| ProposedTask {
                name: t.name,
                description: t.description,
                task_type: parse_task_type(&t.task_type),
                position: t.position,
                depends_on: t.depends_on,
            })
            .collect();

        Ok(tasks)
    }

    /// Run all grooming domain roles, collecting questions from each.
    async fn generate_grooming_questions(
        &self,
        context: &GroomingContext,
        decisions: &[QaDecision],
    ) -> Vec<Question> {
        let mut all_questions = Vec::new();
        for role in prompts::grooming::GROOMING_ROLES {
            let prompt = prompts::grooming::build_grooming_prompt(role, context, decisions);
            match self.generate_qa(&prompt).await {
                Ok(round) => all_questions.extend(round.questions),
                Err(e) => warn!(role = role.id, %e, "grooming role generation failed, skipping"),
            }
        }
        all_questions
    }

    /// Spawn an implementation subprocess and start a stdout-monitoring task.
    async fn spawn_impl(
        &self,
        task_id: Uuid,
        session_id: &str,
        prompt: &str,
        resume: bool,
        worktree: Option<&std::path::Path>,
    ) -> Result<Child> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("--print").arg("--session-id").arg(session_id);
        if resume {
            cmd.arg("--resume");
        }
        if let Some(dir) = worktree {
            cmd.current_dir(dir);
        }
        cmd.arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| anyhow!("spawn failed: {e}"))?;
        let stdout: ChildStdout = child.stdout.take().expect("stdout piped");

        tokio::spawn(monitor_impl_stdout(
            task_id,
            stdout,
            self.dispatch_tx.clone(),
            self.monitor_tx.clone(),
        ));

        Ok(child)
    }

    // ── Misc ──────────────────────────────────────────────────────────────────

    async fn send(&self, msg: DispatchMessage) {
        if self.dispatch_tx.send(msg).await.is_err() {
            error!("dispatcher channel closed");
        }
    }
}

// ── Free helper functions ─────────────────────────────────────────────────────

/// Monitor stdout of an implementation subprocess, forwarding status updates and
/// signalling pause/completion/failure back to the actor via `monitor_tx`.
async fn monitor_impl_stdout(
    task_id: Uuid,
    stdout: ChildStdout,
    dispatch_tx: mpsc::Sender<DispatchMessage>,
    monitor_tx: mpsc::Sender<ImplResult>,
) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut awaiting_decision_json = false;

    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                if line == "[DECISION_NEEDED]" {
                    awaiting_decision_json = true;
                    continue;
                }

                if awaiting_decision_json {
                    awaiting_decision_json = false;
                    match serde_json::from_str::<PauseQuestion>(&line) {
                        Ok(question) => {
                            let _ = monitor_tx
                                .send(ImplResult::Paused { task_id, question })
                                .await;
                            return;
                        }
                        Err(e) => {
                            warn!(%task_id, %e, "could not parse decision JSON '{}'; continuing", line);
                        }
                    }
                    continue;
                }

                if let Some(sha) = line.strip_prefix("[COMPLETED]") {
                    let _ = monitor_tx
                        .send(ImplResult::Completed {
                            task_id,
                            commit_sha: sha.trim().into(),
                        })
                        .await;
                    return;
                }

                // Regular progress output.
                let _ = dispatch_tx
                    .send(DispatchMessage::StatusUpdate {
                        task_id,
                        text: line,
                    })
                    .await;
            }
            Ok(None) => {
                // stdout closed without an explicit marker.
                let _ = monitor_tx
                    .send(ImplResult::Failed {
                        task_id,
                        error: "process ended without [COMPLETED] marker".into(),
                    })
                    .await;
                return;
            }
            Err(e) => {
                let _ = monitor_tx
                    .send(ImplResult::Failed {
                        task_id,
                        error: format!("stdout read error: {e}"),
                    })
                    .await;
                return;
            }
        }
    }
}

/// Convert answers to QaDecisions by looking up matching question text/domain.
pub(crate) fn append_decisions(
    decisions: &mut Vec<QaDecision>,
    questions: &[Question],
    answers: &[Answer],
) {
    for answer in answers {
        if let Some(q) = questions.iter().find(|q| q.id == answer.question_id) {
            decisions.push(QaDecision {
                question_text: q.text.clone(),
                answer_text: answer.selected_answer_text.clone(),
                domain: q.domain.clone(),
            });
        }
    }
}

/// Strip markdown code fences that a model may wrap JSON in.
pub(crate) fn strip_markdown_fences(s: &str) -> &str {
    let s = s.trim();
    let after_open = s
        .strip_prefix("```json")
        .or_else(|| s.strip_prefix("```"))
        .map(str::trim_start);
    if let Some(inner) = after_open
        && let Some(before_close) = inner.strip_suffix("```")
    {
        return before_close.trim();
    }
    s
}

/// Parse the plain-text convergence response from the model.
pub(crate) fn parse_convergence_response(response: &str) -> bool {
    response.contains("SUFFICIENT")
}

fn parse_task_type(s: &str) -> TaskType {
    match s {
        "test" => TaskType::Test,
        "design" => TaskType::Design,
        _ => TaskType::Code,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    // ── strip_markdown_fences ────────────────────────────────────────────────

    #[test]
    fn strip_json_fences() {
        let input = "```json\n{\"questions\":[]}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"questions\":[]}");
    }

    #[test]
    fn strip_plain_fences() {
        let input = "```\n{\"questions\":[]}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"questions\":[]}");
    }

    #[test]
    fn no_fences_unchanged() {
        let input = "{\"questions\":[]}";
        assert_eq!(strip_markdown_fences(input), "{\"questions\":[]}");
    }

    #[test]
    fn strip_fences_with_trailing_whitespace() {
        let input = "```json\n  {\"questions\":[]}  \n```";
        assert_eq!(strip_markdown_fences(input), "{\"questions\":[]}");
    }

    // ── parse_convergence_response ───────────────────────────────────────────

    #[test]
    fn sufficient_returns_true() {
        assert!(parse_convergence_response("SUFFICIENT"));
        assert!(parse_convergence_response("  SUFFICIENT  "));
        assert!(parse_convergence_response(
            "Yes, I have enough info. SUFFICIENT."
        ));
    }

    #[test]
    fn continue_returns_false() {
        assert!(!parse_convergence_response("CONTINUE"));
        assert!(!parse_convergence_response("Need more info. CONTINUE."));
        assert!(!parse_convergence_response(""));
    }

    // ── append_decisions ─────────────────────────────────────────────────────

    fn make_question(id: Uuid, text: &str, domain: &str) -> Question {
        Question {
            id,
            text: text.into(),
            domain: domain.into(),
            rationale: "Test rationale.".into(),
            options: vec![],
            recommended_option_index: 0,
        }
    }

    fn make_answer(question_id: Uuid, answer_text: &str) -> Answer {
        Answer {
            question_id,
            selected_answer_index: Some(0),
            selected_answer_text: answer_text.into(),
            answered_by: Uuid::new_v4(),
            answered_at: Utc::now(),
        }
    }

    #[test]
    fn answers_mapped_to_decisions() {
        let qid = Uuid::now_v7();
        let questions = vec![make_question(qid, "Use REST or GraphQL?", "development")];
        let answers = vec![make_answer(qid, "REST")];
        let mut decisions = vec![];
        append_decisions(&mut decisions, &questions, &answers);

        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].question_text, "Use REST or GraphQL?");
        assert_eq!(decisions[0].answer_text, "REST");
        assert_eq!(decisions[0].domain, "development");
    }

    #[test]
    fn unknown_question_id_ignored() {
        let questions = vec![];
        let answers = vec![make_answer(Uuid::new_v4(), "something")];
        let mut decisions = vec![];
        append_decisions(&mut decisions, &questions, &answers);
        assert!(decisions.is_empty());
    }

    #[test]
    fn multiple_answers_accumulated() {
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        let questions = vec![
            make_question(id1, "DB type?", "planning"),
            make_question(id2, "Cache?", "planning"),
        ];
        let answers = vec![make_answer(id1, "PostgreSQL"), make_answer(id2, "Redis")];
        let mut decisions = vec![];
        append_decisions(&mut decisions, &questions, &answers);
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].answer_text, "PostgreSQL");
        assert_eq!(decisions[1].answer_text, "Redis");
    }

    // ── parse_task_type ──────────────────────────────────────────────────────

    #[test]
    fn task_type_parsing() {
        assert!(matches!(parse_task_type("test"), TaskType::Test));
        assert!(matches!(parse_task_type("design"), TaskType::Design));
        assert!(matches!(parse_task_type("code"), TaskType::Code));
        assert!(matches!(parse_task_type("unknown"), TaskType::Code));
    }

    // ── RawQaRound deserialization ───────────────────────────────────────────

    #[test]
    fn deserialize_raw_qa_round() {
        let json = r#"{"questions":[{"text":"Q?","domain":"business","rationale":"Important.","recommended_option_index":0,"options":[{"label":"A","pros":"Good.","cons":"Bad."},{"label":"B","pros":"Fast.","cons":"Slow."}]}]}"#;
        let raw: RawQaRound = serde_json::from_str(json).unwrap();
        assert_eq!(raw.questions.len(), 1);
        assert_eq!(raw.questions[0].text, "Q?");
        assert_eq!(raw.questions[0].options.len(), 2);
        assert_eq!(raw.questions[0].options[0].label, "A");
        assert_eq!(raw.questions[0].rationale, "Important.");
        assert_eq!(raw.questions[0].recommended_option_index, 0);
    }

    #[test]
    fn deserialize_raw_decomposition() {
        let json = r#"{"tasks":[{"name":"T","description":"D","task_type":"code","position":1,"depends_on":[]}]}"#;
        let raw: RawDecomposition = serde_json::from_str(json).unwrap();
        assert_eq!(raw.tasks.len(), 1);
        assert_eq!(raw.tasks[0].task_type, "code");
    }

    // ── Integration: dispatch routing ────────────────────────────────────────

    #[tokio::test]
    async fn impl_result_completed_routes_to_dispatch() {
        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(4);
        let (_claude_tx, claude_rx) = mpsc::channel(4);
        let mut actor = ClaudeCode::new("claude".into(), dispatch_tx, claude_rx);

        let task_id = Uuid::now_v7();
        actor
            .handle_impl_result(ImplResult::Completed {
                task_id,
                commit_sha: "abc123".into(),
            })
            .await;

        let msg = dispatch_rx.try_recv().unwrap();
        assert!(
            matches!(msg, DispatchMessage::TaskCompleted { commit_sha, .. } if commit_sha == "abc123")
        );
    }

    #[tokio::test]
    async fn impl_result_failed_routes_to_dispatch() {
        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(4);
        let (_claude_tx, claude_rx) = mpsc::channel(4);
        let mut actor = ClaudeCode::new("claude".into(), dispatch_tx, claude_rx);

        let task_id = Uuid::now_v7();
        actor
            .handle_impl_result(ImplResult::Failed {
                task_id,
                error: "crash".into(),
            })
            .await;

        let msg = dispatch_rx.try_recv().unwrap();
        assert!(matches!(msg, DispatchMessage::TaskFailed { error, .. } if error == "crash"));
    }

    #[tokio::test]
    async fn impl_result_paused_routes_to_dispatch() {
        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(4);
        let (_claude_tx, claude_rx) = mpsc::channel(4);
        let mut actor = ClaudeCode::new("claude".into(), dispatch_tx, claude_rx);

        let task_id = Uuid::now_v7();
        actor
            .handle_impl_result(ImplResult::Paused {
                task_id,
                question: PauseQuestion {
                    text: "Which approach?".into(),
                    domain: "development".into(),
                    rationale: "Affects implementation complexity.".into(),
                    options: vec![
                        shared::types::QuestionOption {
                            label: "A".into(),
                            pros: "Simple.".into(),
                            cons: "Limited.".into(),
                        },
                        shared::types::QuestionOption {
                            label: "B".into(),
                            pros: "Flexible.".into(),
                            cons: "Complex.".into(),
                        },
                    ],
                    recommended_option_index: 0,
                },
            })
            .await;

        let msg = dispatch_rx.try_recv().unwrap();
        assert!(matches!(msg, DispatchMessage::TaskPaused { .. }));
    }
}
