use uuid::Uuid;

use shared::{
    messages::{ContainerToServer, ServerToContainer},
    types::{KnowledgeEntry, QaDecision},
};

use crate::{
    container_keys::types::ContainerKeyInfo,
    db::{OrgTx, new_id},
    errors::ApiError,
    qa::{
        repo as qa_repo,
        types::{QaContent, QaQuestion},
    },
    sse::broadcaster::SseEvent,
    state::AppState,
    story::repo as story_repo,
    task::repo as task_repo,
};

use super::{prompts, registry::ConnectionRegistry, session_repo};

// ── Incoming message dispatch ─────────────────────────────────────────────────

/// Route an authenticated `ContainerToServer` message to the appropriate handler.
/// Errors are logged but not propagated — the WS read loop must not abort on a
/// single bad message.
pub async fn handle_message(state: &AppState, key_info: &ContainerKeyInfo, msg: ContainerToServer) {
    let result = match msg {
        ContainerToServer::ExecutionResult { session_id, output } => {
            on_execution_result(state, key_info, session_id, output).await
        }
        ContainerToServer::ExecutionFailed { session_id, error } => {
            on_execution_failed(state, key_info, session_id, &error).await
        }
        ContainerToServer::Pong => Ok(()),
    };

    if let Err(e) = result {
        tracing::error!("container message handler error: {e}");
    }
}


// ── Incoming handlers ─────────────────────────────────────────────────────────

async fn on_execution_result(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session_id: Uuid,
    output: serde_json::Value,
) -> Result<(), ApiError> {
    let session = session_repo::load_session(&state.pool, session_id)
        .await?
        .ok_or_else(|| {
            ApiError::Internal(anyhow::anyhow!(
                "no session found for session_id={session_id}"
            ))
        })?;

    let stage = session.stage.as_str();
    tracing::info!(
        session_id = %session.session_id,
        stage,
        story_id = ?session.story_id,
        task_id = ?session.task_id,
        "execution result received"
    );

    match stage {
        "grooming" | "planning" => on_qa_result(state, key_info, &session, output).await,
        "description_refinement" => on_refinement_result(state, key_info, &session, output).await,
        "decomposition" => on_decomposition_result(state, key_info, &session, output).await,
        "task_qa" => on_task_qa_result(state, key_info, &session, output).await,
        "implementation" => on_implementation_result(state, key_info, &session, output).await,
        other => {
            tracing::warn!(session_id = %session.session_id, stage = %other, "unknown session stage");
            Ok(())
        }
    }
}

/// Handle Q&A results from grooming or planning stages.
///
/// Grooming uses a sequential chain: role 0 produces `{"questions":[...]}`,
/// roles 1-N produce `{"augmentations":[...],"questions":[...]}`.  Each role
/// merges its output into the shared round then triggers the next role until
/// the chain is exhausted, at which point the round is broadcast.
async fn on_qa_result(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session: &session_repo::AgentSession,
    output: serde_json::Value,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let story_id = session.story_id.ok_or(ApiError::NotFound)?;
    let stage = &session.stage;

    let output = normalize_json_output(output)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to normalise QA output: {e}")))?;

    if let Some(qa_round_id) = session.qa_round_id {
        // ── Sequential-grooming path ──────────────────────────────────────────
        // Resolve which roles are enabled for this project, then find the
        // current role's position within that filtered list.
        let role_id = session.role.as_deref().unwrap_or("");
        let project_role_ids = fetch_project_grooming_roles(&state.pool, session.project_id).await?;
        let enabled_roles = enabled_grooming_roles(&project_role_ids);
        let pos_in_enabled = enabled_roles
            .iter()
            .position(|r| r.id == role_id)
            .ok_or_else(|| {
                ApiError::Internal(anyhow::anyhow!("grooming role '{role_id}' not in enabled list for project"))
            })?;

        // Load the current accumulated questions from the shared round.
        let mut tx = OrgTx::begin(&state.pool, org_id).await?;
        let round = qa_repo::get_round(&mut tx, qa_round_id, org_id)
            .await?
            .ok_or_else(|| {
                ApiError::Internal(anyhow::anyhow!("qa_round {qa_round_id} not found"))
            })?;
        tx.commit().await?;

        let mut content: QaContent = serde_json::from_value(round.content)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse round content: {e}")))?;

        if pos_in_enabled == 0 {
            // First enabled role: simple `{"questions":[...]}` format.
            let round_output: QaRoundOutput = serde_json::from_value(output)
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse QA output: {e}")))?;
            let new_questions = output_to_qa_questions(round_output.questions);
            content.questions.extend(new_questions);
        } else {
            // Subsequent role: `{"augmentations":[...],"questions":[...]}` format.
            let seq_output: SequentialQaRoundOutput = serde_json::from_value(output)
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse sequential QA output: {e}")))?;

            apply_augmentations(&mut content.questions, seq_output.augmentations);
            content.questions.extend(output_to_qa_questions(seq_output.questions));
        }

        // Persist the updated content.
        let content_json = serde_json::to_value(&content)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to serialise QA content: {e}")))?;
        let mut tx = OrgTx::begin(&state.pool, org_id).await?;
        qa_repo::update_round_content(&mut tx, qa_round_id, org_id, &content_json)
            .await?
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("qa_round {qa_round_id} not found during update")))?;
        tx.commit().await?;

        if let Some(&next_role) = enabled_roles.get(pos_in_enabled + 1) {
            // Dispatch the next enabled role with the accumulated questions as context.
            dispatch_next_grooming_role(
                state,
                org_id,
                session.project_id,
                story_id,
                qa_round_id,
                next_role,
                &content.questions,
            )
            .await;
        } else {
            // All enabled roles done — broadcast or converge.
            if content.questions.is_empty() {
                dispatch_description_refinement(state, org_id, session.project_id, story_id, "grooming").await;
            } else {
                state.broadcaster.broadcast(
                    org_id,
                    SseEvent::NewQuestion {
                        story_id,
                        task_id: None,
                        round_id: qa_round_id,
                    },
                );
            }
        }
    } else {
        // ── Single-role path (planning, etc.) ─────────────────────────────────
        let round_output: QaRoundOutput = serde_json::from_value(output)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse QA output: {e}")))?;
        let questions = output_to_qa_questions(round_output.questions);

        if questions.is_empty() {
            // No further questions → converge → description refinement
            dispatch_description_refinement(state, org_id, session.project_id, story_id, stage).await;
        } else {
            // Create a fresh round for this response.
            let mut tx = OrgTx::begin(&state.pool, key_info.org_id).await?;

            let max_round = qa_repo::get_max_round_number(&mut tx, story_id, None, stage)
                .await?
                .unwrap_or(0);

            let content_value = serde_json::to_value(&QaContent {
                questions,
                course_correction: None,
            })
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to serialise QA content: {e}")))?;

            let row = qa_repo::create_round(
                &mut tx,
                org_id,
                story_id,
                None,
                stage,
                max_round + 1,
                &content_value,
            )
            .await?;

            tx.commit().await?;

            state.broadcaster.broadcast(
                org_id,
                SseEvent::NewQuestion {
                    story_id,
                    task_id: None,
                    round_id: row.id,
                },
            );
        }
    }

    Ok(())
}

/// Handle refined description output (plain text).
async fn on_refinement_result(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session: &session_repo::AgentSession,
    output: serde_json::Value,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let story_id = session.story_id.ok_or(ApiError::NotFound)?;
    let stage = session.role.as_deref().unwrap_or("grooming");

    let refined_description = output.as_str().unwrap_or("").trim().to_string();

    let mut tx = OrgTx::begin(&state.pool, key_info.org_id).await?;

    story_repo::set_pending_refined_description(&mut tx, story_id, org_id, &refined_description)
        .await?
        .ok_or(ApiError::NotFound)?;

    tx.commit().await?;

    state.broadcaster.broadcast(
        org_id,
        SseEvent::RefinedDescriptionReady {
            story_id,
            stage: stage.to_string(),
        },
    );

    Ok(())
}

/// Handle decomposition output: `{ "tasks": [...] }`
async fn on_decomposition_result(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session: &session_repo::AgentSession,
    output: serde_json::Value,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let story_id = session.story_id.ok_or(ApiError::NotFound)?;

    let output = normalize_json_output(output)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to normalise decomposition output: {e}")))?;
    let decomposition: DecompositionOutput = serde_json::from_value(output)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse decomposition: {e}")))?;

    let mut tx = OrgTx::begin(&state.pool, key_info.org_id).await?;

    let mut task_ids: Vec<Uuid> = Vec::with_capacity(decomposition.tasks.len());
    for pt in &decomposition.tasks {
        let task_type_str = match pt.task_type.as_str() {
            "design" => "design",
            "test" => "test",
            _ => "code",
        };
        let row = task_repo::create_task(
            &mut tx,
            org_id,
            story_id,
            &pt.name,
            &pt.description,
            task_type_str,
            pt.position,
            None,
        )
        .await?;
        task_ids.push(row.id);
    }

    for (i, pt) in decomposition.tasks.iter().enumerate() {
        for &dep_idx in &pt.depends_on {
            let dep_idx = dep_idx as usize;
            if dep_idx < task_ids.len() && dep_idx != i {
                task_repo::create_dependency(&mut tx, task_ids[i], task_ids[dep_idx]).await?;
            }
        }
    }

    story_repo::update_story(
        &mut tx,
        story_id,
        org_id,
        None,
        None,
        None,
        None,
        Some("decomposition"),
    )
    .await?;

    tx.commit().await?;

    state.broadcaster.broadcast(
        org_id,
        SseEvent::StoryUpdated {
            story_id,
            fields: vec!["tasks".into(), "pipeline_stage".into()],
        },
    );

    Ok(())
}

/// Handle task Q&A output (same format as grooming/planning but scoped to a task).
async fn on_task_qa_result(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session: &session_repo::AgentSession,
    output: serde_json::Value,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let story_id = session.story_id.ok_or(ApiError::NotFound)?;
    let task_id = session.task_id.ok_or(ApiError::NotFound)?;

    let output = normalize_json_output(output)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to normalise task QA output: {e}")))?;
    let round: QaRoundOutput = serde_json::from_value(output)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse task QA output: {e}")))?;

    if round.questions.is_empty() {
        // No questions needed → skip directly to implementation.
        dispatch_implementation(state, org_id, session.project_id, story_id, task_id).await;
        return Ok(());
    }

    let mut tx = OrgTx::begin(&state.pool, key_info.org_id).await?;

    // Mark task as paused
    task_repo::update_task(
        &mut tx,
        task_id,
        org_id,
        None,
        None,
        None,
        None,
        Some("paused"),
        None,
        Some("Awaiting Q&A answers"),
    )
    .await?;

    let max_round = qa_repo::get_max_round_number(&mut tx, story_id, Some(task_id), "task_qa")
        .await?
        .unwrap_or(0);

    let questions: Vec<QaQuestion> = round
        .questions
        .into_iter()
        .map(|q| QaQuestion {
            id: new_id(),
            text: q.text,
            domain: q.domain,
            rationale: q.rationale,
            options: q
                .options
                .into_iter()
                .map(|o| crate::qa::types::QaQuestionOption {
                    label: o.label,
                    pros: o.pros,
                    cons: o.cons,
                })
                .collect(),
            recommended_option_index: q.recommended_option_index,
            selected_answer_index: None,
            selected_answer_text: None,
            answered_by: None,
            answered_at: None,
        })
        .collect();

    let content_value = serde_json::to_value(&QaContent {
        questions,
        course_correction: None,
    })
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to serialise QA content: {e}")))?;

    let row = qa_repo::create_round(
        &mut tx,
        org_id,
        story_id,
        Some(task_id),
        "task_qa",
        max_round + 1,
        &content_value,
    )
    .await?;

    tx.commit().await?;

    state.broadcaster.broadcast(
        org_id,
        SseEvent::TaskUpdated {
            task_id,
            story_id,
            fields: vec!["state".into(), "ai_status_text".into()],
        },
    );
    state.broadcaster.broadcast(
        org_id,
        SseEvent::NewQuestion {
            story_id,
            task_id: Some(task_id),
            round_id: row.id,
        },
    );

    Ok(())
}

/// Handle implementation result: `{ "commit_sha": "..." }` or `{ "error": "..." }`
async fn on_implementation_result(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session: &session_repo::AgentSession,
    output: serde_json::Value,
) -> Result<(), ApiError> {
    let _org_id = key_info.org_id;
    let task_id = session.task_id.ok_or(ApiError::NotFound)?;

    if let Some(error) = output.get("error").and_then(|v| v.as_str()) {
        on_task_failed(state, key_info, task_id, error).await
    } else if let Some(sha) = output.get("commit_sha").and_then(|v| v.as_str()) {
        on_task_completed(state, key_info, task_id, sha).await
    } else {
        tracing::warn!(task_id = %task_id, "implementation result has neither commit_sha nor error");
        Ok(())
    }
}

async fn on_execution_failed(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session_id: Uuid,
    error: &str,
) -> Result<(), ApiError> {
    let session = match session_repo::load_session(&state.pool, session_id).await? {
        Some(s) => s,
        None => {
            tracing::error!(%session_id, "execution failed but no session found");
            return Ok(());
        }
    };

    match session.stage.as_str() {
        "implementation" | "task_qa" => {
            if let Some(task_id) = session.task_id {
                on_task_failed(state, key_info, task_id, error).await?;
            }
        }
        stage => {
            tracing::error!(
                %session_id,
                %stage,
                %error,
                "execution failed for non-task stage"
            );
        }
    }

    Ok(())
}

// ── Task state helpers (reused by multiple handlers) ──────────────────────────

async fn on_task_completed(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    task_id: Uuid,
    commit_sha: &str,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let mut tx = OrgTx::begin(&state.pool, key_info.org_id).await?;

    let story_id = task_repo::get_task(&mut tx, task_id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?
        .story_id;

    let status_text = format!("Completed — awaiting review (sha: {commit_sha})");
    task_repo::update_task(
        &mut tx,
        task_id,
        org_id,
        None,
        None,
        None,
        None,
        Some("running"),
        None,
        Some(&status_text),
    )
    .await?;

    let all_tasks = task_repo::list_tasks(&mut tx, story_id).await?;
    let all_reviewed = !all_tasks.is_empty()
        && all_tasks
            .iter()
            .all(|t| t.state == "running" || t.state == "done");

    if all_reviewed {
        story_repo::update_story(
            &mut tx,
            story_id,
            org_id,
            None,
            None,
            None,
            None,
            Some("review"),
        )
        .await?;
    }

    tx.commit().await?;

    state
        .broadcaster
        .broadcast(org_id, SseEvent::TaskCompleted { task_id, story_id });

    if all_reviewed {
        state.broadcaster.broadcast(
            org_id,
            SseEvent::StoryUpdated {
                story_id,
                fields: vec!["pipeline_stage".into()],
            },
        );
    }

    Ok(())
}

async fn on_task_failed(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    task_id: Uuid,
    error: &str,
) -> Result<(), ApiError> {
    let mut tx = OrgTx::begin(&state.pool, key_info.org_id).await?;
    let org_id = tx.org_id;

    let story_id = task_repo::get_task(&mut tx, task_id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?
        .story_id;

    task_repo::update_task(
        &mut tx,
        task_id,
        org_id,
        None,
        None,
        None,
        None,
        Some("blocked"),
        None,
        Some(error),
    )
    .await?;

    tx.commit().await?;

    state.broadcaster.broadcast(
        org_id,
        SseEvent::TaskUpdated {
            task_id,
            story_id,
            fields: vec!["state".into(), "ai_status_text".into()],
        },
    );

    Ok(())
}

// ── Outgoing dispatch functions ───────────────────────────────────────────────

/// Dispatch the first grooming Q&A round for a story — all roles in parallel,
/// all contributing questions into the same shared `qa_round`.
pub async fn dispatch_grooming(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
    description: &str,
) {
    tracing::info!(%story_id, %project_id, "dispatching grooming (sequential roles)");

    let round_id = match create_shared_grooming_round(&state.pool, org_id, story_id).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(%story_id, %e, "failed to create shared grooming round");
            return;
        }
    };

    let knowledge = fetch_knowledge(&state.pool, org_id, project_id)
        .await
        .unwrap_or_default();

    // Only dispatch role 0 (business analyst); subsequent roles are chained in on_qa_result.
    let role = &prompts::grooming::GROOMING_ROLES[0];
    let (system_prompt, prompt) =
        prompts::grooming::build_grooming_prompt(role, description, &knowledge, "", &[]);

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "grooming",
        Some(role.id),
        Some(round_id),
    )
    .await
    {
        Ok(session_id) => {
            send_execute(state, project_id, session_id, &system_prompt, &prompt).await;
        }
        Err(e) => tracing::error!(%story_id, role = role.id, %e, "failed to create grooming session"),
    }
}

/// Dispatch planning Q&A for a story.
pub async fn dispatch_planning(state: &AppState, org_id: Uuid, project_id: Uuid, story_id: Uuid) {
    tracing::info!(%story_id, %project_id, "dispatching planning");
    let knowledge = fetch_knowledge(&state.pool, org_id, project_id)
        .await
        .unwrap_or_default();

    let description = fetch_story_description(&state.pool, org_id, story_id)
        .await
        .unwrap_or_default();

    let grooming_decisions = fetch_stage_decisions(&state.pool, org_id, story_id, "grooming")
        .await
        .unwrap_or_default();

    let planning_decisions = fetch_stage_decisions(&state.pool, org_id, story_id, "planning")
        .await
        .unwrap_or_default();

    let (system_prompt, prompt) = prompts::planning::build_planning_prompt(
        &description,
        &knowledge,
        "",
        &grooming_decisions,
        &planning_decisions,
    );

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "planning",
        None,
        None,
    )
    .await
    {
        Ok(session_id) => {
            send_execute(state, project_id, session_id, &system_prompt, &prompt).await;
        }
        Err(e) => tracing::error!(%story_id, %e, "failed to create planning session"),
    }
}

/// Dispatch description refinement after convergence.
async fn dispatch_description_refinement(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
    stage: &str,
) {
    tracing::info!(%story_id, %project_id, stage, "dispatching description refinement");
    let description = fetch_story_description(&state.pool, org_id, story_id)
        .await
        .unwrap_or_default();

    let decisions = fetch_stage_decisions(&state.pool, org_id, story_id, stage)
        .await
        .unwrap_or_default();

    let (system_prompt, prompt) =
        prompts::description_refinement::build_refinement_prompt(&description, &decisions, stage);

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "description_refinement",
        Some(stage),
        None,
    )
    .await
    {
        Ok(session_id) => {
            send_execute(state, project_id, session_id, &system_prompt, &prompt).await;
        }
        Err(e) => tracing::error!(%story_id, %e, "failed to create refinement session"),
    }
}

/// Dispatch decomposition after planning is complete.
pub async fn dispatch_decomposition(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
) {
    tracing::info!(%story_id, %project_id, "dispatching task decomposition");
    let description = fetch_story_description(&state.pool, org_id, story_id)
        .await
        .unwrap_or_default();

    let grooming_decisions = fetch_stage_decisions(&state.pool, org_id, story_id, "grooming")
        .await
        .unwrap_or_default();
    let planning_decisions = fetch_stage_decisions(&state.pool, org_id, story_id, "planning")
        .await
        .unwrap_or_default();

    let (system_prompt, prompt) = prompts::task_decomposition::build_decomposition_prompt(
        &description,
        "",
        &grooming_decisions,
        &planning_decisions,
    );

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "decomposition",
        None,
        None,
    )
    .await
    {
        Ok(session_id) => {
            send_execute(state, project_id, session_id, &system_prompt, &prompt).await;
        }
        Err(e) => tracing::error!(%story_id, %e, "failed to create decomposition session"),
    }
}

/// Dispatch the next round of Q&A after all answers are submitted.
/// Re-dispatches the same stage so the LLM can self-converge (return empty
/// questions when it has no further questions).
pub async fn dispatch_next_round(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
    stage: &str,
    task_id: Option<Uuid>,
) {
    if let Some(tid) = task_id {
        // Task-level Q&A: dispatch implementation with the new answers.
        dispatch_implementation(state, org_id, project_id, story_id, tid).await;
    } else {
        // Story-level Q&A: re-dispatch the same stage for the next round.
        match stage {
            "grooming" => dispatch_next_grooming_round(state, org_id, project_id, story_id).await,
            "planning" => dispatch_planning(state, org_id, project_id, story_id).await,
            _ => {}
        }
    }
}

/// Dispatch implementation for a specific task.
pub async fn dispatch_implementation(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
    task_id: Uuid,
) {
    tracing::info!(%task_id, %story_id, %project_id, "dispatching implementation");
    let knowledge = fetch_knowledge(&state.pool, org_id, project_id)
        .await
        .unwrap_or_default();

    let story_decisions = fetch_all_story_decisions(&state.pool, org_id, story_id)
        .await
        .unwrap_or_default();

    let task_description = fetch_task_description(&state.pool, org_id, task_id)
        .await
        .unwrap_or_default();

    let (system_prompt, prompt) = prompts::implementation::build_implementation_prompt(
        &task_description,
        &knowledge,
        &story_decisions,
        &[], // sibling decisions not tracked yet
    );

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        Some(task_id),
        "implementation",
        None,
        None,
    )
    .await
    {
        Ok(session_id) => {
            send_execute(state, project_id, session_id, &system_prompt, &prompt).await;
        }
        Err(e) => tracing::error!(%task_id, %e, "failed to create implementation session"),
    }
}

/// Dispatch the next grooming round — all roles in parallel, with existing decisions as context.
async fn dispatch_next_grooming_round(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
) {
    tracing::info!(%story_id, %project_id, "dispatching next grooming round (sequential roles)");

    let round_id = match create_shared_grooming_round(&state.pool, org_id, story_id).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(%story_id, %e, "failed to create shared grooming round");
            return;
        }
    };

    let description = fetch_story_description(&state.pool, org_id, story_id)
        .await
        .unwrap_or_default();

    let knowledge = fetch_knowledge(&state.pool, org_id, project_id)
        .await
        .unwrap_or_default();

    let decisions = fetch_stage_decisions(&state.pool, org_id, story_id, "grooming")
        .await
        .unwrap_or_default();

    // Only dispatch role 0; the chain continues in on_qa_result.
    let role = &prompts::grooming::GROOMING_ROLES[0];
    let (system_prompt, prompt) =
        prompts::grooming::build_grooming_prompt(role, &description, &knowledge, "", &decisions);

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "grooming",
        Some(role.id),
        Some(round_id),
    )
    .await
    {
        Ok(session_id) => {
            send_execute(state, project_id, session_id, &system_prompt, &prompt).await;
        }
        Err(e) => tracing::error!(%story_id, role = role.id, %e, "failed to create next grooming session"),
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Send an `Execute` message to the container connected for the given project.
async fn send_execute(
    state: &AppState,
    project_id: Uuid,
    session_id: Uuid,
    system_prompt: &str,
    prompt: &str,
) {
    match find_connected_key(&state.pool, state.registry.as_ref(), project_id).await {
        Some(key_id) => {
            tracing::info!(%project_id, %session_id, "sending Execute to container");
            let _ = state.registry.send_to(
                key_id,
                ServerToContainer::Execute {
                    session_id,
                    system_prompt: system_prompt.to_string(),
                    prompt: prompt.to_string(),
                },
            );
        }
        None => {
            tracing::warn!(
                %project_id,
                %session_id,
                "no container agent connected for project — execute dropped; \
                 ensure the agent is running and connected via /ws/container"
            );
        }
    }
}

/// Return the first `key_id` from the candidates that has a live WebSocket connection.
fn find_key_in_registry(ids: &[Uuid], registry: &dyn ConnectionRegistry) -> Option<Uuid> {
    ids.iter().copied().find(|&id| registry.is_connected(id))
}

/// Look up non-revoked keys for a project (pool-level query, no RLS needed) and
/// return the first one that is currently connected in the registry.
async fn find_connected_key(
    pool: &sqlx::PgPool,
    registry: &dyn ConnectionRegistry,
    project_id: Uuid,
) -> Option<Uuid> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM container_api_keys WHERE project_id = $1 AND revoked_at IS NULL",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let ids: Vec<Uuid> = rows.into_iter().map(|(id,)| id).collect();
    find_key_in_registry(&ids, registry)
}

/// Fetch the enabled grooming role IDs for a project (pool-level, no RLS needed).
async fn fetch_project_grooming_roles(
    pool: &sqlx::PgPool,
    project_id: Uuid,
) -> Result<Vec<String>, ApiError> {
    let row: (Vec<String>,) = sqlx::query_as(
        "SELECT grooming_roles FROM projects WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Filter `GROOMING_ROLES` to only those whose IDs appear in `role_ids`,
/// preserving the canonical order.
fn enabled_grooming_roles(
    role_ids: &[String],
) -> Vec<&'static prompts::grooming::GroomingRole> {
    prompts::grooming::GROOMING_ROLES
        .iter()
        .filter(|r| role_ids.iter().any(|id| id.as_str() == r.id))
        .collect()
}

/// Fetch knowledge entries (org + project level) without RLS — uses pool directly.
async fn fetch_knowledge(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<Vec<KnowledgeEntry>, ApiError> {
    let rows = sqlx::query_as::<_, crate::knowledge::repo::KnowledgeRow>(
        r#"
        SELECT id, org_id, project_id, story_id, category, title, content, created_at, updated_at
        FROM knowledge_entries
        WHERE deleted_at IS NULL
          AND org_id = $1
          AND (
              (project_id IS NULL AND story_id IS NULL)
              OR (project_id = $2 AND story_id IS NULL)
          )
        ORDER BY created_at
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(to_knowledge_entries(rows))
}

fn to_knowledge_entries(rows: Vec<crate::knowledge::repo::KnowledgeRow>) -> Vec<KnowledgeEntry> {
    rows.into_iter()
        .map(|r| KnowledgeEntry {
            title: r.title,
            content: r.content,
            category: parse_knowledge_category(&r.category),
        })
        .collect()
}

fn parse_knowledge_category(s: &str) -> shared::enums::KnowledgeCategory {
    use shared::enums::KnowledgeCategory;
    match s {
        "convention" => KnowledgeCategory::Convention,
        "adr" => KnowledgeCategory::Adr,
        "api_doc" => KnowledgeCategory::ApiDoc,
        "design_system" => KnowledgeCategory::DesignSystem,
        _ => KnowledgeCategory::Custom,
    }
}

/// Extract `QaDecision`s from persisted rounds.
fn extract_decisions(rounds: &[crate::qa::repo::QaRoundRow]) -> Result<Vec<QaDecision>, ApiError> {
    let mut decisions = Vec::new();
    for round in rounds {
        let content: QaContent = serde_json::from_value(round.content.clone())
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("bad QA content: {e}")))?;
        for q in content.questions {
            if let (Some(text), Some(domain)) = (q.selected_answer_text, Some(q.domain)) {
                decisions.push(QaDecision {
                    question_text: q.text,
                    answer_text: text,
                    domain,
                });
            }
        }
    }
    Ok(decisions)
}

/// Convert LLM output questions (no id) into stored `QaQuestion` records.
fn output_to_qa_questions(questions: Vec<QaQuestionOutput>) -> Vec<QaQuestion> {
    questions
        .into_iter()
        .map(|q| QaQuestion {
            id: new_id(),
            text: q.text,
            domain: q.domain,
            rationale: q.rationale,
            options: q
                .options
                .into_iter()
                .map(|o| crate::qa::types::QaQuestionOption {
                    label: o.label,
                    pros: o.pros,
                    cons: o.cons,
                })
                .collect(),
            recommended_option_index: q.recommended_option_index,
            selected_answer_index: None,
            selected_answer_text: None,
            answered_by: None,
            answered_at: None,
        })
        .collect()
}

/// Merge augmentation output from a subsequent grooming role into the accumulated question list.
/// Each augmentation appends the role's perspective to the rationale and each option's pros/cons.
fn apply_augmentations(questions: &mut Vec<QaQuestion>, augmentations: Vec<QaAugmentationOutput>) {
    for aug in augmentations {
        let Some(q) = questions.get_mut(aug.question_index) else {
            tracing::warn!(
                question_index = aug.question_index,
                "augmentation references out-of-bounds question index — skipping"
            );
            continue;
        };

        if !aug.rationale_addition.trim().is_empty() {
            q.rationale = format!("{}\n\n{}", q.rationale, aug.rationale_addition.trim());
        }

        for (i, opt_aug) in aug.options.into_iter().enumerate() {
            let Some(opt) = q.options.get_mut(i) else { break };
            if !opt_aug.pros_addition.trim().is_empty() {
                opt.pros = format!("{}\n\n{}", opt.pros, opt_aug.pros_addition.trim());
            }
            if !opt_aug.cons_addition.trim().is_empty() {
                opt.cons = format!("{}\n\n{}", opt.cons, opt_aug.cons_addition.trim());
            }
        }
    }
}

/// Dispatch a single grooming role, passing the accumulated questions
/// from all previous roles in the chain as context.
async fn dispatch_next_grooming_role(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
    qa_round_id: Uuid,
    role: &'static prompts::grooming::GroomingRole,
    accumulated_questions: &[QaQuestion],
) {
    tracing::info!(%story_id, role = role.id, "dispatching sequential grooming role");

    let description = fetch_story_description(&state.pool, org_id, story_id)
        .await
        .unwrap_or_default();

    let knowledge = fetch_knowledge(&state.pool, org_id, project_id)
        .await
        .unwrap_or_default();

    let decisions = fetch_stage_decisions(&state.pool, org_id, story_id, "grooming")
        .await
        .unwrap_or_default();

    let acc: Vec<prompts::grooming::AccumulatedQuestion<'_>> = accumulated_questions
        .iter()
        .enumerate()
        .map(|(i, q)| prompts::grooming::AccumulatedQuestion {
            index: i,
            text: &q.text,
            domain: &q.domain,
            rationale: &q.rationale,
            options: q.options.iter().map(|o| (o.label.as_str(), o.pros.as_str(), o.cons.as_str())).collect(),
        })
        .collect();

    let (system_prompt, prompt) =
        prompts::grooming::build_sequential_grooming_prompt(role, &description, &knowledge, "", &decisions, &acc);

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "grooming",
        Some(role.id),
        Some(qa_round_id),
    )
    .await
    {
        Ok(session_id) => {
            send_execute(state, project_id, session_id, &system_prompt, &prompt).await;
        }
        Err(e) => tracing::error!(%story_id, role = role.id, %e, "failed to create sequential grooming session"),
    }
}

#[cfg(test)]
fn into_qa_questions(questions: Vec<shared::types::Question>) -> Vec<QaQuestion> {
    questions
        .into_iter()
        .map(|q| QaQuestion {
            id: q.id,
            text: q.text,
            domain: q.domain,
            rationale: q.rationale,
            options: q
                .options
                .into_iter()
                .map(|o| crate::qa::types::QaQuestionOption {
                    label: o.label,
                    pros: o.pros,
                    cons: o.cons,
                })
                .collect(),
            recommended_option_index: q.recommended_option_index,
            selected_answer_index: None,
            selected_answer_text: None,
            answered_by: None,
            answered_at: None,
        })
        .collect()
}

/// Fetch the story description directly (pool-level, no RLS).
async fn fetch_story_description(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<String, ApiError> {
    let row: (String,) = sqlx::query_as(
        "SELECT description FROM stories WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL",
    )
    .bind(story_id)
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Fetch all decisions for a specific stage of a story.
async fn fetch_stage_decisions(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    story_id: Uuid,
    stage: &str,
) -> Result<Vec<QaDecision>, ApiError> {
    let mut tx = OrgTx::begin(pool, org_id).await?;
    let rounds = qa_repo::list_rounds(&mut tx, org_id, Some(story_id), None, Some(stage)).await?;
    let decisions = extract_decisions(&rounds)?;
    tx.commit().await?;
    Ok(decisions)
}

/// Fetch all story-level decisions (grooming + planning combined).
async fn fetch_all_story_decisions(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<Vec<QaDecision>, ApiError> {
    let mut grooming = fetch_stage_decisions(pool, org_id, story_id, "grooming").await?;
    let planning = fetch_stage_decisions(pool, org_id, story_id, "planning").await?;
    grooming.extend(planning);
    Ok(grooming)
}

/// Fetch task description directly (pool-level, no RLS).
async fn fetch_task_description(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    task_id: Uuid,
) -> Result<String, ApiError> {
    let row: (String,) = sqlx::query_as(
        "SELECT COALESCE(description, name) FROM tasks WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL",
    )
    .bind(task_id)
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Pre-create an empty shared grooming round that all parallel roles will
/// append their questions into.  Returns the new round's `id`.
async fn create_shared_grooming_round(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<Uuid, ApiError> {
    let mut tx = OrgTx::begin(pool, org_id).await?;

    let max_round = qa_repo::get_max_round_number(&mut tx, story_id, None, "grooming")
        .await?
        .unwrap_or(0);

    let empty_content = serde_json::json!({ "questions": [], "course_correction": null });
    let row = qa_repo::create_round(
        &mut tx,
        org_id,
        story_id,
        None,
        "grooming",
        max_round + 1,
        &empty_content,
    )
    .await?;

    tx.commit().await?;
    Ok(row.id)
}

// ── JSON normalisation ────────────────────────────────────────────────────────

/// Strip markdown code fences that Claude sometimes wraps JSON output in.
fn strip_markdown_fences(s: &str) -> &str {
    let s = s.trim();
    let s = if let Some(rest) = s.strip_prefix("```json") {
        rest
    } else if let Some(rest) = s.strip_prefix("```") {
        rest
    } else {
        return s;
    };
    s.strip_suffix("```").unwrap_or(s).trim()
}

/// Normalise a `serde_json::Value` coming from Claude.
///
/// Claude sometimes wraps its JSON output in a markdown code fence, which
/// causes the `result` field of the `--output-format json` envelope to be a
/// plain string rather than a parsed object.  This helper unwraps that string
/// so callers can always use `serde_json::from_value`.
fn normalize_json_output(output: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    if output.is_object() || output.is_array() {
        return Ok(output);
    }
    if let Some(s) = output.as_str() {
        let clean = strip_markdown_fences(s);
        let parsed: serde_json::Value = serde_json::from_str(clean)
            .map_err(|e| anyhow::anyhow!("failed to parse inner JSON from string output: {e}"))?;
        return Ok(parsed);
    }
    Ok(output)
}

// ── JSON parse types for agent output ─────────────────────────────────────────

#[derive(serde::Deserialize)]
struct QaRoundOutput {
    questions: Vec<QaQuestionOutput>,
}

/// Output format for roles 1-N in the sequential grooming chain.
#[derive(serde::Deserialize)]
struct SequentialQaRoundOutput {
    #[serde(default)]
    augmentations: Vec<QaAugmentationOutput>,
    #[serde(default)]
    questions: Vec<QaQuestionOutput>,
}

#[derive(serde::Deserialize)]
struct QaAugmentationOutput {
    question_index: usize,
    #[serde(default)]
    rationale_addition: String,
    options: Vec<QaOptionAugmentationOutput>,
}

#[derive(serde::Deserialize)]
struct QaOptionAugmentationOutput {
    #[serde(default)]
    pros_addition: String,
    #[serde(default)]
    cons_addition: String,
}

#[derive(serde::Deserialize)]
struct QaQuestionOutput {
    text: String,
    domain: String,
    rationale: String,
    options: Vec<QaQuestionOptionOutput>,
    recommended_option_index: usize,
}

#[derive(serde::Deserialize)]
struct QaQuestionOptionOutput {
    label: String,
    pros: String,
    cons: String,
}

#[derive(serde::Deserialize)]
struct DecompositionOutput {
    tasks: Vec<DecompositionTask>,
}

#[derive(serde::Deserialize)]
struct DecompositionTask {
    name: String,
    description: String,
    task_type: String,
    position: i32,
    #[serde(default)]
    depends_on: Vec<i32>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;
    use uuid::Uuid;

    use shared::{messages::ServerToContainer, types::Question};

    use crate::agents::registry::{ConnectionRegistry, DashMapRegistry};

    use super::{find_key_in_registry, into_qa_questions};

    fn make_registry() -> DashMapRegistry {
        DashMapRegistry::new()
    }

    fn register_key(reg: &DashMapRegistry) -> (Uuid, mpsc::UnboundedReceiver<ServerToContainer>) {
        let id = Uuid::now_v7();
        let (tx, rx) = mpsc::unbounded_channel::<ServerToContainer>();
        reg.register(id, tx);
        (id, rx)
    }

    // ── into_qa_questions ─────────────────────────────────────────────────────

    #[test]
    fn into_qa_questions_preserves_fields() {
        let id = Uuid::now_v7();
        let q = Question {
            id,
            text: "What?".into(),
            domain: "design".into(),
            rationale: "Important for design.".into(),
            options: vec![
                shared::types::QuestionOption {
                    label: "A".into(),
                    pros: "Good.".into(),
                    cons: "Bad.".into(),
                },
                shared::types::QuestionOption {
                    label: "B".into(),
                    pros: "Fast.".into(),
                    cons: "Slow.".into(),
                },
            ],
            recommended_option_index: 0,
        };
        let result = into_qa_questions(vec![q]);
        assert_eq!(result.len(), 1);
        let qa = &result[0];
        assert_eq!(qa.id, id);
        assert_eq!(qa.text, "What?");
        assert_eq!(qa.domain, "design");
        assert_eq!(qa.rationale, "Important for design.");
        assert_eq!(qa.options.len(), 2);
        assert_eq!(qa.options[0].label, "A");
        assert_eq!(qa.recommended_option_index, 0);
        assert!(qa.selected_answer_index.is_none());
        assert!(qa.selected_answer_text.is_none());
        assert!(qa.answered_by.is_none());
        assert!(qa.answered_at.is_none());
    }

    #[test]
    fn into_qa_questions_empty() {
        assert!(into_qa_questions(vec![]).is_empty());
    }

    // ── find_key_in_registry ──────────────────────────────────────────────────

    #[test]
    fn find_key_no_candidates() {
        let reg = make_registry();
        assert!(find_key_in_registry(&[], &reg).is_none());
    }

    #[test]
    fn find_key_none_connected() {
        let reg = make_registry();
        let ids = vec![Uuid::now_v7(), Uuid::now_v7()];
        assert!(find_key_in_registry(&ids, &reg).is_none());
    }

    #[test]
    fn find_key_returns_connected_key() {
        let reg = make_registry();
        let (id, _rx) = register_key(&reg);
        let result = find_key_in_registry(&[id], &reg);
        assert_eq!(result, Some(id));
    }

    #[test]
    fn find_key_skips_unconnected_returns_first_connected() {
        let reg = make_registry();
        let unconnected = Uuid::now_v7(); // never registered
        let (connected, _rx) = register_key(&reg);
        let result = find_key_in_registry(&[unconnected, connected], &reg);
        assert_eq!(result, Some(connected));
    }

    #[test]
    fn find_key_receiver_dropped_counts_as_disconnected() {
        let reg = make_registry();
        let (id, rx) = register_key(&reg);
        drop(rx); // simulate container disconnect
        assert!(find_key_in_registry(&[id], &reg).is_none());
    }

    // ── outgoing message format ───────────────────────────────────────────────

    #[test]
    fn registry_send_to_delivers_ping() {
        let reg = make_registry();
        let (id, mut rx) = register_key(&reg);
        reg.send_to(id, ServerToContainer::Ping).unwrap();
        assert!(matches!(rx.try_recv().unwrap(), ServerToContainer::Ping));
    }
}
