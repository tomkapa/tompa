use uuid::Uuid;

use shared::{
    messages::{ContainerToServer, ServerToContainer},
    types::{Answer, GroomingContext, PauseQuestion, PlanningContext, QaRoundContent, TaskContext},
};

use crate::{
    auth::middleware::set_org_context,
    container_keys::types::ContainerKeyInfo,
    db::new_id,
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

use super::registry::ConnectionRegistry;

// ── Incoming message dispatch ─────────────────────────────────────────────────

/// Route an authenticated `ContainerToServer` message to the appropriate handler.
/// Errors are logged but not propagated — the WS read loop must not abort on a
/// single bad message.
pub async fn handle_message(state: &AppState, key_info: &ContainerKeyInfo, msg: ContainerToServer) {
    let result = match msg {
        ContainerToServer::QuestionBatch { story_id, task_id, round } => {
            on_question_batch(state, key_info, story_id, task_id, round).await
        }
        ContainerToServer::TaskDecomposition { story_id, proposed_tasks } => {
            on_task_decomposition(state, key_info, story_id, proposed_tasks).await
        }
        ContainerToServer::TaskPaused { task_id, question } => {
            on_task_paused(state, key_info, task_id, question).await
        }
        ContainerToServer::TaskCompleted { task_id, commit_sha } => {
            on_task_completed(state, key_info, task_id, &commit_sha).await
        }
        ContainerToServer::TaskFailed { task_id, error } => {
            on_task_failed(state, key_info, task_id, &error).await
        }
        ContainerToServer::StatusUpdate { task_id, status_text } => {
            on_status_update(state, key_info, task_id, &status_text).await
        }
        ContainerToServer::Pong => Ok(()),
    };

    if let Err(e) = result {
        eprintln!("container message handler error: {e}");
    }
}

// ── Incoming handlers (private) ───────────────────────────────────────────────

async fn on_question_batch(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    story_id: Uuid,
    task_id: Option<Uuid>,
    round: QaRoundContent,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    // Determine stage from context: task-scoped rounds are always "task_qa";
    // story-level rounds inherit the story's current pipeline_stage.
    let stage = if task_id.is_some() {
        "task_qa".to_string()
    } else {
        story_repo::get_story(&mut tx, story_id)
            .await?
            .ok_or(ApiError::NotFound)?
            .pipeline_stage
            .unwrap_or_else(|| "grooming".to_string())
    };

    let max_round = qa_repo::get_max_round_number(&mut tx, story_id, task_id, &stage)
        .await?
        .unwrap_or(0);

    let content_value = serde_json::to_value(&QaContent {
        questions: into_qa_questions(round.questions),
        course_correction: None,
    })
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to serialise QA content: {e}")))?;

    let row = qa_repo::create_round(
        &mut tx,
        org_id,
        story_id,
        task_id,
        &stage,
        max_round + 1,
        &content_value,
    )
    .await?;

    tx.commit().await?;

    state
        .broadcaster
        .broadcast(org_id, SseEvent::NewQuestion { story_id, task_id, round_id: row.id });

    Ok(())
}

async fn on_task_decomposition(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    story_id: Uuid,
    proposed_tasks: Vec<shared::types::ProposedTask>,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    // Insert tasks in proposal order and keep the assigned IDs for dependency wiring.
    let mut task_ids: Vec<Uuid> = Vec::with_capacity(proposed_tasks.len());
    for pt in &proposed_tasks {
        let task_type_str = match pt.task_type {
            shared::enums::TaskType::Design => "design",
            shared::enums::TaskType::Test => "test",
            shared::enums::TaskType::Code => "code",
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

    // Wire dependency edges: `depends_on` holds 0-based indices into proposed_tasks.
    for (i, pt) in proposed_tasks.iter().enumerate() {
        for &dep_idx in &pt.depends_on {
            let dep_idx = dep_idx as usize;
            if dep_idx < task_ids.len() && dep_idx != i {
                task_repo::create_dependency(&mut tx, task_ids[i], task_ids[dep_idx]).await?;
            }
        }
    }

    story_repo::update_story(&mut tx, story_id, None, None, None, None, Some("decomposition"))
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

async fn on_task_paused(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    task_id: Uuid,
    question: PauseQuestion,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let story_id = task_repo::get_task(&mut tx, task_id)
        .await?
        .ok_or(ApiError::NotFound)?
        .story_id;

    // Mark task as paused; store the question text as the status message.
    task_repo::update_task(
        &mut tx,
        task_id,
        None,
        None,
        None,
        None,
        Some("paused"),
        None,
        Some(&question.text),
    )
    .await?;

    let max_round =
        qa_repo::get_max_round_number(&mut tx, story_id, Some(task_id), "task_qa")
            .await?
            .unwrap_or(0);

    let content_value = serde_json::to_value(&QaContent {
        questions: vec![QaQuestion {
            id: new_id(),
            text: question.text,
            domain: question.domain,
            options: question.options,
            selected_answer_index: None,
            selected_answer_text: None,
            answered_by: None,
            answered_at: None,
        }],
        course_correction: None,
    })
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to serialise QA content: {e}")))?;

    let round = qa_repo::create_round(
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
        SseEvent::NewQuestion { story_id, task_id: Some(task_id), round_id: round.id },
    );

    Ok(())
}

async fn on_task_completed(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    task_id: Uuid,
    commit_sha: &str,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let story_id = task_repo::get_task(&mut tx, task_id)
        .await?
        .ok_or(ApiError::NotFound)?
        .story_id;

    // State stays "running" — human must call /done to mark it complete.
    // The commit SHA is embedded in ai_status_text so no schema change is required.
    let status_text = format!("Completed — awaiting review (sha: {commit_sha})");
    task_repo::update_task(
        &mut tx,
        task_id,
        None,
        None,
        None,
        None,
        Some("running"),
        None,
        Some(&status_text),
    )
    .await?;

    // If every task for the story is now at "running" (awaiting review) or "done",
    // advance the story's pipeline to "review".
    let all_tasks = task_repo::list_tasks(&mut tx, story_id).await?;
    let all_reviewed = !all_tasks.is_empty()
        && all_tasks.iter().all(|t| t.state == "running" || t.state == "done");

    if all_reviewed {
        story_repo::update_story(&mut tx, story_id, None, None, None, None, Some("review"))
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
    let org_id = key_info.org_id;
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let story_id = task_repo::get_task(&mut tx, task_id)
        .await?
        .ok_or(ApiError::NotFound)?
        .story_id;

    task_repo::update_task(
        &mut tx,
        task_id,
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

async fn on_status_update(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    task_id: Uuid,
    status_text: &str,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let story_id = task_repo::get_task(&mut tx, task_id)
        .await?
        .ok_or(ApiError::NotFound)?
        .story_id;

    task_repo::update_task(
        &mut tx,
        task_id,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(status_text),
    )
    .await?;

    tx.commit().await?;

    state.broadcaster.broadcast(
        org_id,
        SseEvent::TaskUpdated {
            task_id,
            story_id,
            fields: vec!["ai_status_text".into()],
        },
    );

    Ok(())
}

// ── Outgoing message triggers ─────────────────────────────────────────────────

/// Send `StartGrooming` to the project's connected container.
/// Called by the story service when a story moves to `in_progress`.
pub async fn send_start_grooming(
    state: &AppState,
    project_id: Uuid,
    story_id: Uuid,
    context: GroomingContext,
) {
    if let Some(key_id) = find_connected_key(&state.pool, state.registry.as_ref(), project_id).await
    {
        let _ =
            state.registry.send_to(key_id, ServerToContainer::StartGrooming { story_id, context });
    }
}

/// Send `StartPlanning` to the project's connected container.
/// Called when grooming Q&A is complete and the AI assessment is SUFFICIENT.
pub async fn send_start_planning(
    state: &AppState,
    project_id: Uuid,
    story_id: Uuid,
    context: PlanningContext,
) {
    if let Some(key_id) = find_connected_key(&state.pool, state.registry.as_ref(), project_id).await
    {
        let _ = state
            .registry
            .send_to(key_id, ServerToContainer::StartPlanning { story_id, context });
    }
}

/// Forward completed answers to the container that originated the QA round.
/// Called by the Q&A service after all questions in a round have been answered.
pub async fn send_answer_received(
    state: &AppState,
    project_id: Uuid,
    round_id: Uuid,
    answers: Vec<Answer>,
) {
    if let Some(key_id) = find_connected_key(&state.pool, state.registry.as_ref(), project_id).await
    {
        let _ = state
            .registry
            .send_to(key_id, ServerToContainer::AnswerReceived { round_id, answers });
    }
}

/// Send `StartTask` to the project's dev container and return the generated session ID.
/// Returns `None` if no container is connected for the project.
/// Called when a task's dependencies are met and it is ready for execution.
pub async fn send_start_task(
    state: &AppState,
    project_id: Uuid,
    story_id: Uuid,
    task_id: Uuid,
    context: TaskContext,
) -> Option<String> {
    let key_id = find_connected_key(&state.pool, state.registry.as_ref(), project_id).await?;
    let session_id = Uuid::now_v7().to_string();
    let _ = state.registry.send_to(
        key_id,
        ServerToContainer::StartTask {
            story_id,
            task_id,
            session_id: session_id.clone(),
            context,
        },
    );
    Some(session_id)
}

/// Send `ResumeTask` to the dev container after a human answers a pause question.
pub async fn send_resume_task(
    state: &AppState,
    project_id: Uuid,
    task_id: Uuid,
    session_id: String,
    answer: Answer,
) {
    if let Some(key_id) = find_connected_key(&state.pool, state.registry.as_ref(), project_id).await
    {
        let _ = state.registry.send_to(
            key_id,
            ServerToContainer::ResumeTask { task_id, session_id, answer },
        );
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Convert the wire-format question list to the server-side `QaQuestion` format.
/// The per-question answer fields start as `None` (unanswered).
fn into_qa_questions(questions: Vec<shared::types::Question>) -> Vec<QaQuestion> {
    questions
        .into_iter()
        .map(|q| QaQuestion {
            id: q.id,
            text: q.text,
            domain: q.domain,
            options: q.options,
            selected_answer_index: None,
            selected_answer_text: None,
            answered_by: None,
            answered_at: None,
        })
        .collect()
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
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("SELECT id FROM container_api_keys WHERE project_id = $1 AND revoked_at IS NULL")
            .bind(project_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let ids: Vec<Uuid> = rows.into_iter().map(|(id,)| id).collect();
    find_key_in_registry(&ids, registry)
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
            options: vec!["A".into(), "B".into()],
        };
        let result = into_qa_questions(vec![q]);
        assert_eq!(result.len(), 1);
        let qa = &result[0];
        assert_eq!(qa.id, id);
        assert_eq!(qa.text, "What?");
        assert_eq!(qa.domain, "design");
        assert_eq!(qa.options, vec!["A", "B"]);
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
