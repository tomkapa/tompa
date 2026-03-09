use uuid::Uuid;

use shared::{
    messages::{ContainerToServer, ServerToContainer},
    types::{KnowledgeEntry, QaDecision},
};

use crate::{
    container_keys::types::ContainerKeyInfo,
    db::{OrgTx, new_id},
    decision_patterns::{
        repo as dp_repo,
        service as dp_service,
        types::{DecisionPatternResponse, ExtractedPattern},
    },
    errors::ApiError,
    project_profiles::{
        repo as pp_repo,
        service as pp_service,
        types::ProjectProfileContent,
    },
    qa::{
        repo as qa_repo,
        types::{AppliedPatternSummary, QaContent, QaQuestion},
    },
    sse::broadcaster::SseEvent,
    state::AppState,
    story::repo as story_repo,
    task::repo as task_repo,
};

use super::{prompts, registry::ConnectionRegistry, session_repo};

// ── Prompt context (knowledge + profile + patterns) ──────────────────────────

/// Combined context for prompt construction.
pub struct PromptContext {
    pub knowledge: Vec<KnowledgeEntry>,
    pub profile: Option<ProjectProfileContent>,
    pub patterns: Vec<DecisionPatternResponse>,
}

/// Fetch all three layers of project intelligence for prompt construction.
async fn fetch_prompt_context(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    project_id: Uuid,
    story_description: &str,
    exclude_story_id: Option<Uuid>,
) -> Result<PromptContext, ApiError> {
    let knowledge = fetch_knowledge(pool, org_id, project_id).await?;
    let profile = pp_service::fetch_project_profile(pool, org_id, project_id).await?;

    // Extract simple keyword tags from the story description for FTS
    let tags: Vec<String> = extract_search_tags(story_description);
    let pattern_rows = dp_repo::fetch_relevant_patterns(pool, org_id, project_id, story_description, &tags, exclude_story_id).await?;
    let patterns = pattern_rows
        .into_iter()
        .map(|r| DecisionPatternResponse {
            id: r.id,
            org_id: r.org_id,
            project_id: r.project_id,
            domain: r.domain,
            pattern: r.pattern,
            rationale: r.rationale,
            tags: r.tags,
            confidence: r.confidence,
            usage_count: r.usage_count,
            override_count: r.override_count,
            source_story_id: r.source_story_id,
            source_round_id: r.source_round_id,
            superseded_by: r.superseded_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect();

    Ok(PromptContext { knowledge, profile, patterns })
}

/// Extract simple keyword tags from text for FTS tag matching.
fn extract_search_tags(text: &str) -> Vec<String> {
    // Extract domain-relevant keywords as tags
    let domain_keywords = [
        "api", "database", "auth", "ui", "ux", "security", "performance",
        "testing", "deployment", "cache", "queue", "async", "sync",
        "rest", "graphql", "websocket", "sse", "pagination", "search",
    ];
    text.split_whitespace()
        .filter_map(|w| {
            let lower = w.to_lowercase();
            let clean = lower.trim_matches(|c: char| !c.is_alphanumeric());
            if domain_keywords.contains(&clean) {
                Some(clean.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Format a project profile for injection into prompts.
fn fmt_profile(profile: &ProjectProfileContent) -> String {
    let mut sections = Vec::new();

    sections.push(format!("**Identity:** {}", profile.identity));

    if !profile.tech_stack.is_empty() {
        let stack: Vec<String> = profile
            .tech_stack
            .iter()
            .map(|(k, v)| format!("- {k}: {v}"))
            .collect();
        sections.push(format!("**Tech Stack:**\n{}", stack.join("\n")));
    }

    if !profile.architectural_patterns.is_empty() {
        let pats: Vec<String> = profile
            .architectural_patterns
            .iter()
            .map(|p| format!("- {p}"))
            .collect();
        sections.push(format!("**Architectural Patterns:**\n{}", pats.join("\n")));
    }

    if !profile.conventions.is_empty() {
        let convs: Vec<String> = profile.conventions.iter().map(|c| format!("- {c}")).collect();
        sections.push(format!("**Conventions:**\n{}", convs.join("\n")));
    }

    if !profile.team_preferences.is_empty() {
        let prefs: Vec<String> = profile
            .team_preferences
            .iter()
            .map(|p| format!("- {p}"))
            .collect();
        sections.push(format!("**Team Preferences:**\n{}", prefs.join("\n")));
    }

    sections.join("\n\n")
}

/// Format decision patterns for injection into prompts.
fn fmt_patterns(patterns: &[DecisionPatternResponse]) -> String {
    if patterns.is_empty() {
        return String::new();
    }
    patterns
        .iter()
        .enumerate()
        .map(|(i, p)| {
            format!(
                "{}. [{}] {} (confidence: {:.0}%)",
                i + 1,
                p.domain,
                p.pattern,
                p.confidence * 100.0
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build the prompt sections for profile and patterns context.
fn build_context_sections(ctx: &PromptContext) -> String {
    let mut sections = String::new();

    if let Some(ref profile) = ctx.profile {
        sections.push_str("\n\n## Project Profile\nThis project's established identity and conventions:\n\n");
        sections.push_str(&fmt_profile(profile));
        sections.push_str("\n\nTreat these as established context. Do not re-ask decisions that align with the profile unless the story explicitly conflicts.");
    }

    let patterns_text = fmt_patterns(&ctx.patterns);
    if !patterns_text.is_empty() {
        sections.push_str("\n\n## Established Project Patterns\nThese patterns were established by prior decisions in this project.\nUse them as defaults unless the story requirements clearly conflict.\nIf a pattern covers a question you'd normally ask, skip it or propose it as the recommended default instead of asking.\n\n");
        sections.push_str(&patterns_text);
    }

    // Add pattern extraction instruction to the JSON output format.
    sections.push_str(r#"

Additionally, extract 0-3 reusable decision patterns from this round's context — abstract principles that would apply to future stories with similar concerns. Include them in your JSON response as:
"patterns": [{"domain": "development|security|design|business|marketing", "pattern": "One-sentence reusable rule", "rationale": "Why this was decided", "tags": ["tag1", "tag2"]}]
If no patterns are worth extracting, return "patterns": []."#);

    sections
}

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
        "profile_synthesis" => on_profile_synthesis_result(state, key_info, &session, output).await,
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
        let qa_config = fetch_project_qa_config(&state.pool, session.project_id).await?;
        let enabled_roles = parse_enabled_grooming_roles(&qa_config);
        let pos_in_enabled = enabled_roles
            .iter()
            .position(|r| r.id == role_id)
            .ok_or_else(|| {
                ApiError::Internal(anyhow::anyhow!(
                    "grooming role '{role_id}' not in enabled list for project"
                ))
            })?;

        // Load the current accumulated questions from the shared round.
        let mut tx = OrgTx::begin(&state.pool, org_id).await?;
        let round = qa_repo::get_round(&mut tx, qa_round_id, org_id)
            .await?
            .ok_or_else(|| {
                ApiError::Internal(anyhow::anyhow!("qa_round {qa_round_id} not found"))
            })?;
        tx.commit().await?;

        let round_number = round.round_number;
        let mut content: QaContent = serde_json::from_value(round.content).map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("failed to parse round content: {e}"))
        })?;

        let extracted_patterns: Vec<ExtractedPattern>;

        if pos_in_enabled == 0 {
            // First enabled role: simple `{"questions":[...]}` format.
            let round_output: QaRoundOutput = serde_json::from_value(output).map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("failed to parse QA output: {e}"))
            })?;
            extracted_patterns = round_output.patterns.into_iter().map(Into::into).collect();
            let new_questions = output_to_qa_questions(round_output.questions);
            content.questions.extend(new_questions);
        } else {
            // Subsequent role: `{"augmentations":[...],"questions":[...]}` format.
            let seq_output: SequentialQaRoundOutput =
                serde_json::from_value(output).map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("failed to parse sequential QA output: {e}"))
                })?;

            extracted_patterns = seq_output.patterns.into_iter().map(Into::into).collect();
            apply_augmentations(&mut content.questions, seq_output.augmentations);
            content
                .questions
                .extend(output_to_qa_questions(seq_output.questions));
        }

        // Process extracted patterns (piggybacked distillation — zero extra LLM calls).
        if !extracted_patterns.is_empty() {
            tracing::info!(
                %org_id, %story_id, count = extracted_patterns.len(),
                "processing extracted patterns from grooming round"
            );
            dp_service::process_extracted_patterns(
                &state.pool, org_id, session.project_id, story_id,
                Some(qa_round_id), extracted_patterns,
            ).await;
        }

        // Persist the updated content.
        let content_json = serde_json::to_value(&content).map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("failed to serialise QA content: {e}"))
        })?;
        let mut tx = OrgTx::begin(&state.pool, org_id).await?;
        qa_repo::update_round_content(&mut tx, qa_round_id, org_id, &content_json)
            .await?
            .ok_or_else(|| {
                ApiError::Internal(anyhow::anyhow!(
                    "qa_round {qa_round_id} not found during update"
                ))
            })?;
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
                &qa_config,
                round_number,
            )
            .await;
        } else {
            // All enabled roles done — broadcast or converge.
            if content.questions.is_empty() {
                dispatch_description_refinement(
                    state,
                    org_id,
                    session.project_id,
                    story_id,
                    "grooming",
                )
                .await;
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

        // Process extracted patterns from planning output.
        let extracted_patterns: Vec<ExtractedPattern> =
            round_output.patterns.into_iter().map(Into::into).collect();
        if !extracted_patterns.is_empty() {
            tracing::info!(
                %org_id, %story_id, stage, count = extracted_patterns.len(),
                "processing extracted patterns from planning round"
            );
            dp_service::process_extracted_patterns(
                &state.pool, org_id, session.project_id, story_id,
                None, extracted_patterns,
            ).await;
        }

        let questions = output_to_qa_questions(round_output.questions);

        if questions.is_empty() {
            // No further questions → converge → description refinement
            dispatch_description_refinement(state, org_id, session.project_id, story_id, stage)
                .await;
        } else {
            // Create a fresh round for this response.
            let mut tx = OrgTx::begin(&state.pool, key_info.org_id).await?;

            let max_round = qa_repo::get_max_round_number(&mut tx, story_id, None, stage)
                .await?
                .unwrap_or(0);

            let content_value = serde_json::to_value(&QaContent {
                questions,
                course_correction: None,
                applied_patterns: Vec::new(),
            })
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("failed to serialise QA content: {e}"))
            })?;

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

    let output = normalize_json_output(output).map_err(|e| {
        ApiError::Internal(anyhow::anyhow!(
            "failed to normalise decomposition output: {e}"
        ))
    })?;
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

    let output = normalize_json_output(output).map_err(|e| {
        ApiError::Internal(anyhow::anyhow!("failed to normalise task QA output: {e}"))
    })?;
    let round: QaRoundOutput = serde_json::from_value(output)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse task QA output: {e}")))?;

    // Process extracted patterns from task QA.
    let extracted_patterns: Vec<ExtractedPattern> =
        round.patterns.into_iter().map(Into::into).collect();
    if !extracted_patterns.is_empty() {
        tracing::info!(
            %org_id, %story_id, %task_id, count = extracted_patterns.len(),
            "processing extracted patterns from task QA round"
        );
        dp_service::process_extracted_patterns(
            &state.pool, org_id, session.project_id, story_id,
            None, extracted_patterns,
        ).await;
    }

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
            assigned_to: None,
        })
        .collect();

    let content_value = serde_json::to_value(&QaContent {
        questions,
        course_correction: None,
        applied_patterns: Vec::new(),
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

/// Handle profile synthesis result: parse JSON profile content and UPSERT to DB.
async fn on_profile_synthesis_result(
    state: &AppState,
    key_info: &ContainerKeyInfo,
    session: &session_repo::AgentSession,
    output: serde_json::Value,
) -> Result<(), ApiError> {
    let org_id = key_info.org_id;
    let project_id = session.project_id;

    tracing::info!(%org_id, %project_id, "profile synthesis result received");

    let output = normalize_json_output(output).map_err(|e| {
        ApiError::Internal(anyhow::anyhow!("failed to normalise profile synthesis output: {e}"))
    })?;

    // Validate the output parses as ProjectProfileContent
    let _profile: ProjectProfileContent = serde_json::from_value(output.clone()).map_err(|e| {
        tracing::error!(%org_id, %project_id, %e, "failed to parse profile synthesis output as ProjectProfileContent");
        ApiError::Internal(anyhow::anyhow!("failed to parse profile synthesis output: {e}"))
    })?;

    // Count current patterns for the snapshot
    let (_, total_count) = dp_repo::count_patterns_for_threshold(&state.pool, org_id, project_id)
        .await
        .unwrap_or((0, 0));

    pp_repo::upsert_profile(
        &state.pool,
        org_id,
        project_id,
        &output,
        total_count as i32,
        "auto",
    )
    .await?;

    tracing::info!(
        %org_id, %project_id,
        patterns_at_generation = total_count,
        "project profile synthesized and stored"
    );

    Ok(())
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

        // Check if profile regeneration should be triggered.
        let project_id = match fetch_story_project_id(&state.pool, org_id, story_id).await {
            Ok(pid) => pid,
            Err(e) => {
                tracing::error!(%story_id, %org_id, %e, "failed to fetch project_id for profile threshold check");
                return Ok(());
            }
        };
        maybe_trigger_profile_synthesis(state, org_id, project_id).await;
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

    let (round_id, round_number) = match create_shared_grooming_round(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %e, "failed to create shared grooming round");
            return;
        }
    };

    let qa_config = match fetch_project_qa_config(&state.pool, project_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch qa config for grooming");
            return;
        }
    };

    let ctx = match fetch_prompt_context(&state.pool, org_id, project_id, description, Some(story_id)).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch prompt context for grooming");
            return;
        }
    };

    // Record which patterns were injected so the Q&A UI can display provenance.
    set_round_applied_patterns(&state.pool, org_id, round_id, &ctx.patterns).await;

    // Only dispatch role 0 (business analyst); subsequent roles are chained in on_qa_result.
    let enabled_roles = parse_enabled_grooming_roles(&qa_config);
    let role = match enabled_roles.first() {
        Some(r) => r,
        None => {
            tracing::error!(%story_id, "no grooming roles enabled — grooming skipped");
            return;
        }
    };
    let (model_short, detail_level, max_questions) = grooming_role_config(&qa_config, &role.id);
    let detail_text = prompts::detail_levels::detail_level_threshold(detail_level);
    let convergence_text = prompts::detail_levels::convergence_guidance(detail_level, round_number);
    let model = prompts::models::resolve_model_id(&model_short);

    let (system_prompt, base_prompt) = prompts::grooming::build_grooming_prompt(
        role,
        description,
        &ctx.knowledge,
        "",
        &[],
        detail_text,
        max_questions,
        round_number,
        0, // no prior decisions on first round
        &convergence_text,
    );
    let prompt = format!("{base_prompt}{}", build_context_sections(&ctx));

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "grooming",
        Some(role.id.as_str()),
        Some(round_id),
    )
    .await
    {
        Ok(session_id) => {
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
        }
        Err(e) => {
            tracing::error!(%story_id, role = %role.id, %e, "failed to create grooming session")
        }
    }
}

/// Dispatch planning Q&A for a story.
pub async fn dispatch_planning(state: &AppState, org_id: Uuid, project_id: Uuid, story_id: Uuid) {
    tracing::info!(%story_id, %project_id, "dispatching planning");

    let qa_config = match fetch_project_qa_config(&state.pool, project_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch qa config for planning");
            return;
        }
    };
    let (model_short, detail_level, max_questions) = stage_config(&qa_config, "planning");
    let detail_text = prompts::detail_levels::detail_level_threshold(detail_level);
    let model = prompts::models::resolve_model_id(&model_short);

    let (story_title, description) = match fetch_story_description(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch story description for planning");
            return;
        }
    };
    let story_context = fmt_story_context(&story_title, &description);

    let ctx = match fetch_prompt_context(&state.pool, org_id, project_id, &story_context, Some(story_id)).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch prompt context for planning");
            return;
        }
    };

    let grooming_decisions = match fetch_stage_decisions(&state.pool, org_id, story_id, "grooming").await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch grooming decisions for planning");
            return;
        }
    };

    let planning_decisions = match fetch_stage_decisions(&state.pool, org_id, story_id, "planning").await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch planning decisions for planning");
            return;
        }
    };

    let (system_prompt, base_prompt) = prompts::planning::build_planning_prompt(
        &story_context,
        &ctx.knowledge,
        "",
        &grooming_decisions,
        &planning_decisions,
        detail_text,
        max_questions,
    );
    let prompt = format!("{base_prompt}{}", build_context_sections(&ctx));

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
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
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
    let (story_title, description) = match fetch_story_description(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, stage, %e, "failed to fetch story description for refinement");
            return;
        }
    };

    let decisions = match fetch_stage_decisions(&state.pool, org_id, story_id, stage).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, stage, %e, "failed to fetch stage decisions for refinement");
            return;
        }
    };

    let (system_prompt, prompt) = prompts::description_refinement::build_refinement_prompt(
        &story_title,
        &description,
        &decisions,
        stage,
    );

    // Description refinement uses a fixed default model (not in qa_config).
    let model = prompts::models::resolve_model_id("sonnet");

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
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
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
    let (story_title, description) = match fetch_story_description(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch story description for decomposition");
            return;
        }
    };
    let story_context = fmt_story_context(&story_title, &description);

    let grooming_decisions = match fetch_stage_decisions(&state.pool, org_id, story_id, "grooming").await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch grooming decisions for decomposition");
            return;
        }
    };
    let planning_decisions = match fetch_stage_decisions(&state.pool, org_id, story_id, "planning").await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch planning decisions for decomposition");
            return;
        }
    };

    let (system_prompt, prompt) = prompts::task_decomposition::build_decomposition_prompt(
        &story_context,
        "",
        &grooming_decisions,
        &planning_decisions,
    );

    // Decomposition uses a fixed default model (not in qa_config).
    let model = prompts::models::resolve_model_id("sonnet");

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
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
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

    let qa_config = match fetch_project_qa_config(&state.pool, project_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%task_id, %story_id, %project_id, %e, "failed to fetch qa config for implementation");
            return;
        }
    };
    let (model_short, _, _) = stage_config(&qa_config, "implementation");
    let model = prompts::models::resolve_model_id(&model_short);

    let task_description = match fetch_task_description(&state.pool, org_id, task_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%task_id, %story_id, %project_id, %e, "failed to fetch task description for implementation");
            return;
        }
    };

    let ctx = match fetch_prompt_context(&state.pool, org_id, project_id, &task_description, Some(story_id)).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%task_id, %story_id, %project_id, %e, "failed to fetch prompt context for implementation");
            return;
        }
    };

    let story_decisions = match fetch_all_story_decisions(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%task_id, %story_id, %project_id, %e, "failed to fetch story decisions for implementation");
            return;
        }
    };

    let (system_prompt, base_prompt) = prompts::implementation::build_implementation_prompt(
        &task_description,
        &ctx.knowledge,
        &story_decisions,
        &[], // sibling decisions not tracked yet
    );
    let prompt = format!("{base_prompt}{}", build_context_sections(&ctx));

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
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
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

    let (round_id, round_number) = match create_shared_grooming_round(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %e, "failed to create shared grooming round");
            return;
        }
    };

    let qa_config = match fetch_project_qa_config(&state.pool, project_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch qa config for next grooming round");
            return;
        }
    };

    let (story_title, description) = match fetch_story_description(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch story description for next grooming round");
            return;
        }
    };
    let story_context = fmt_story_context(&story_title, &description);

    let ctx = match fetch_prompt_context(&state.pool, org_id, project_id, &story_context, Some(story_id)).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch prompt context for next grooming round");
            return;
        }
    };

    // Record which patterns were injected so the Q&A UI can display provenance.
    set_round_applied_patterns(&state.pool, org_id, round_id, &ctx.patterns).await;

    let decisions = match fetch_stage_decisions(&state.pool, org_id, story_id, "grooming").await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, %project_id, %e, "failed to fetch grooming decisions for next grooming round");
            return;
        }
    };

    // Only dispatch role 0 (BA); the chain continues in on_qa_result.
    let enabled_roles = parse_enabled_grooming_roles(&qa_config);
    let role = match enabled_roles.first() {
        Some(r) => r,
        None => {
            tracing::error!(%story_id, "no grooming roles enabled — next grooming round skipped");
            return;
        }
    };
    let (model_short, detail_level, max_questions) = grooming_role_config(&qa_config, &role.id);
    let detail_text = prompts::detail_levels::detail_level_threshold(detail_level);
    let convergence_text = prompts::detail_levels::convergence_guidance(detail_level, round_number);
    let model = prompts::models::resolve_model_id(&model_short);

    let (system_prompt, base_prompt) = prompts::grooming::build_grooming_prompt(
        role,
        &story_context,
        &ctx.knowledge,
        "",
        &decisions,
        detail_text,
        max_questions,
        round_number,
        decisions.len(),
        &convergence_text,
    );
    let prompt = format!("{base_prompt}{}", build_context_sections(&ctx));

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "grooming",
        Some(role.id.as_str()),
        Some(round_id),
    )
    .await
    {
        Ok(session_id) => {
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
        }
        Err(e) => {
            tracing::error!(%story_id, role = %role.id, %e, "failed to create next grooming session")
        }
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
    model: &str,
) {
    match find_connected_key(&state.pool, state.registry.as_ref(), project_id).await {
        Some(key_id) => {
            tracing::info!(%project_id, %session_id, %model, "sending Execute to container");
            let _ = state.registry.send_to(
                key_id,
                ServerToContainer::Execute {
                    session_id,
                    system_prompt: system_prompt.to_string(),
                    prompt: prompt.to_string(),
                    model: model.to_string(),
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
    let rows: Vec<(Uuid,)> = match sqlx::query_as(
        "SELECT id FROM container_api_keys WHERE project_id = $1 AND revoked_at IS NULL",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%project_id, %e, "failed to fetch container api keys");
            return None;
        }
    };

    let ids: Vec<Uuid> = rows.into_iter().map(|(id,)| id).collect();
    find_key_in_registry(&ids, registry)
}

/// Fetch the qa_config JSON for a project (pool-level, no RLS needed).
async fn fetch_project_qa_config(
    pool: &sqlx::PgPool,
    project_id: Uuid,
) -> Result<serde_json::Value, ApiError> {
    let row: (serde_json::Value,) =
        sqlx::query_as("SELECT qa_config FROM projects WHERE id = $1 AND deleted_at IS NULL")
            .bind(project_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

/// Filter grooming roles to those present in `qa_config.grooming`, preserving canonical order.
fn parse_enabled_grooming_roles(
    qa_config: &serde_json::Value,
) -> Vec<&'static prompts::grooming::GroomingRole> {
    let grooming = qa_config.get("grooming").and_then(|v| v.as_object());
    let Some(grooming) = grooming else {
        return vec![];
    };
    prompts::grooming::GROOMING_CONFIG
        .roles
        .iter()
        .filter(|role| grooming.contains_key(role.id.as_str()))
        .collect()
}

/// Extract (model_short_id, detail_level, max_questions) for a grooming role from qa_config.
fn grooming_role_config(qa_config: &serde_json::Value, role_id: &str) -> (String, i64, i64) {
    let cfg = qa_config.get("grooming").and_then(|g| g.get(role_id));
    extract_role_config(cfg)
}

/// Extract (model_short_id, detail_level, max_questions) for planning or implementation.
fn stage_config(qa_config: &serde_json::Value, stage: &str) -> (String, i64, i64) {
    extract_role_config(qa_config.get(stage))
}

fn extract_role_config(cfg: Option<&serde_json::Value>) -> (String, i64, i64) {
    let model = cfg
        .and_then(|c| c.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("sonnet")
        .to_string();
    let detail_level = cfg
        .and_then(|c| c.get("detail_level"))
        .and_then(|v| v.as_i64())
        .unwrap_or(3);
    let max_questions = cfg
        .and_then(|c| c.get("max_questions"))
        .and_then(|v| v.as_i64())
        .unwrap_or(3);
    (model, detail_level, max_questions)
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
            assigned_to: None,
        })
        .collect()
}

/// Merge augmentation output from a subsequent grooming role into the accumulated question list.
/// Each augmentation appends the role's perspective to the rationale and each option's pros/cons.
fn apply_augmentations(questions: &mut [QaQuestion], augmentations: Vec<QaAugmentationOutput>) {
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
            let Some(opt) = q.options.get_mut(i) else {
                break;
            };
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
#[allow(clippy::too_many_arguments)]
async fn dispatch_next_grooming_role(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
    qa_round_id: Uuid,
    role: &'static prompts::grooming::GroomingRole,
    accumulated_questions: &[QaQuestion],
    qa_config: &serde_json::Value,
    round_number: i32,
) {
    tracing::info!(%story_id, role = %role.id, "dispatching sequential grooming role");

    let (model_short, detail_level, max_questions) = grooming_role_config(qa_config, &role.id);
    let detail_text = prompts::detail_levels::detail_level_threshold(detail_level);
    let convergence_text = prompts::detail_levels::convergence_guidance(detail_level, round_number);
    let model = prompts::models::resolve_model_id(&model_short);

    let (story_title, description) = match fetch_story_description(&state.pool, org_id, story_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, role = %role.id, %e, "failed to fetch story description for sequential grooming role");
            return;
        }
    };
    let story_context = fmt_story_context(&story_title, &description);

    let ctx = match fetch_prompt_context(&state.pool, org_id, project_id, &story_context, Some(story_id)).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, role = %role.id, %e, "failed to fetch prompt context for sequential grooming role");
            return;
        }
    };

    let decisions = match fetch_stage_decisions(&state.pool, org_id, story_id, "grooming").await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%story_id, role = %role.id, %e, "failed to fetch grooming decisions for sequential grooming role");
            return;
        }
    };

    let acc: Vec<prompts::grooming::AccumulatedQuestion<'_>> = accumulated_questions
        .iter()
        .enumerate()
        .map(|(i, q)| prompts::grooming::AccumulatedQuestion {
            index: i,
            text: &q.text,
            domain: &q.domain,
            rationale: &q.rationale,
            options: q
                .options
                .iter()
                .map(|o| (o.label.as_str(), o.pros.as_str(), o.cons.as_str()))
                .collect(),
        })
        .collect();

    let (system_prompt, base_prompt) = prompts::grooming::build_sequential_grooming_prompt(
        role,
        &story_context,
        &ctx.knowledge,
        "",
        &decisions,
        &acc,
        detail_text,
        max_questions,
        round_number,
        decisions.len(),
        &convergence_text,
    );
    let prompt = format!("{base_prompt}{}", build_context_sections(&ctx));

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        Some(story_id),
        None,
        "grooming",
        Some(role.id.as_str()),
        Some(qa_round_id),
    )
    .await
    {
        Ok(session_id) => {
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
        }
        Err(e) => {
            tracing::error!(%story_id, role = %role.id, %e, "failed to create sequential grooming session")
        }
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
            assigned_to: None,
        })
        .collect()
}

/// Fetch the story title and description directly (pool-level, no RLS).
/// Returns `(title, description)`.
async fn fetch_story_description(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<(String, String), ApiError> {
    let row: (String, String) = sqlx::query_as(
        "SELECT title, description FROM stories WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL",
    )
    .bind(story_id)
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    Ok((row.0, row.1))
}

/// Fetch the project_id for a story (pool-level, no RLS).
async fn fetch_story_project_id(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<Uuid, ApiError> {
    let row: (Uuid,) = sqlx::query_as(
        "SELECT project_id FROM stories WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL",
    )
    .bind(story_id)
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Check threshold and dispatch profile synthesis if needed.
async fn maybe_trigger_profile_synthesis(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
) {
    let (new_count, total_count) = match dp_repo::count_patterns_for_threshold(
        &state.pool, org_id, project_id,
    ).await {
        Ok(counts) => counts,
        Err(e) => {
            tracing::error!(%org_id, %project_id, %e, "failed to count patterns for threshold");
            return;
        }
    };

    if total_count == 0 {
        return; // No patterns to synthesize
    }

    let profile_exists = match pp_repo::get_profile_by_project(&state.pool, org_id, project_id).await {
        Ok(p) => p.is_some(),
        Err(e) => {
            tracing::error!(%org_id, %project_id, %e, "failed to check profile existence");
            return;
        }
    };

    let ratio = new_count as f64 / total_count as f64;
    let should_generate =
        ratio >= 0.30 || (total_count >= 3 && !profile_exists);

    if !should_generate {
        tracing::debug!(
            %org_id, %project_id, new_count, total_count, ratio,
            "profile generation threshold not met"
        );
        return;
    }

    tracing::info!(
        %org_id, %project_id, new_count, total_count, ratio,
        "profile generation threshold met — dispatching synthesis"
    );

    dispatch_profile_synthesis(state, org_id, project_id, total_count as i32).await;
}

/// Dispatch profile synthesis via the container agent.
pub async fn dispatch_profile_synthesis(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    total_patterns: i32,
) {
    tracing::info!(%org_id, %project_id, total_patterns, "dispatching profile synthesis");

    // Fetch all active patterns for the project
    let patterns = match sqlx::query_as::<_, dp_repo::DecisionPatternRow>(
        r#"
        SELECT id, org_id, project_id, domain, pattern, rationale, tags, confidence,
               usage_count, override_count, source_story_id, source_round_id,
               superseded_by, created_at, updated_at
        FROM decision_patterns
        WHERE org_id = $1 AND project_id = $2
          AND superseded_by IS NULL AND deleted_at IS NULL
        ORDER BY confidence DESC
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .fetch_all(&state.pool)
    .await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%org_id, %project_id, %e, "failed to fetch patterns for synthesis");
            return;
        }
    };

    let patterns_json: Vec<serde_json::Value> = patterns
        .iter()
        .map(|p| {
            serde_json::json!({
                "domain": p.domain,
                "pattern": p.pattern,
                "rationale": p.rationale,
                "tags": p.tags,
                "confidence": p.confidence,
            })
        })
        .collect();

    let knowledge = match fetch_knowledge(&state.pool, org_id, project_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%org_id, %project_id, %e, "failed to fetch knowledge for synthesis");
            return;
        }
    };

    let knowledge_text: String = knowledge
        .iter()
        .map(|k| format!("- [{:?}] {}: {}", k.category, k.title, k.content))
        .collect::<Vec<_>>()
        .join("\n");

    let current_profile = match pp_service::fetch_project_profile(&state.pool, org_id, project_id).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(%org_id, %project_id, %e, "failed to fetch current profile for synthesis");
            None
        }
    };

    let current_profile_text = match &current_profile {
        Some(p) => serde_json::to_string_pretty(p).unwrap_or_else(|_| "null".to_string()),
        None => "null".to_string(),
    };

    let system_prompt = "You are synthesizing a project intelligence profile from decision patterns \
and knowledge entries. This profile will be injected into future AI prompts \
to provide holistic project context.".to_string();

    let prompt = format!(
        r#"## Decision Patterns (from past Q&A)
{patterns_json}

## Knowledge Entries (documented conventions)
{knowledge}

## Current Profile (if any — update, don't start from scratch)
{current_profile}

Synthesize the above into a structured project profile. Merge overlapping
information. Resolve contradictions by favoring higher-confidence patterns
and more recent entries.

Respond ONLY with valid JSON matching this schema:
{{
  "identity": "One sentence describing what this project is",
  "tech_stack": {{ "layer": "technologies" }},
  "architectural_patterns": ["pattern 1", "pattern 2"],
  "conventions": ["convention 1", "convention 2"],
  "team_preferences": ["preference 1", "preference 2"],
  "domain_knowledge": ["insight 1", "insight 2"]
}}

Rules:
- Each array item should be one concise sentence
- Deduplicate — no two items should say the same thing differently
- tech_stack keys should be logical layers (backend, frontend, database, etc.)
- If the current profile exists, preserve user edits where they don't
  conflict with newer patterns"#,
        patterns_json = serde_json::to_string_pretty(&patterns_json).unwrap_or_default(),
        knowledge = knowledge_text,
        current_profile = current_profile_text,
    );

    let model = prompts::models::resolve_model_id("sonnet");

    match session_repo::create_session(
        &state.pool,
        org_id,
        project_id,
        None,
        None,
        "profile_synthesis",
        None,
        None,
    )
    .await
    {
        Ok(session_id) => {
            send_execute(
                state,
                project_id,
                session_id,
                &system_prompt,
                &prompt,
                model,
            )
            .await;
        }
        Err(e) => tracing::error!(%org_id, %project_id, %e, "failed to create profile synthesis session"),
    }
}

/// Build a combined story context string for use in prompts.
/// Always includes the title; appends description if non-empty.
fn fmt_story_context(title: &str, description: &str) -> String {
    let description = description.trim();
    if description.is_empty() {
        title.to_owned()
    } else {
        format!("{title}\n\n{description}")
    }
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

/// Record which decision patterns were injected into a Q&A round's prompt.
/// Called after `fetch_prompt_context` resolves, so the patterns are known.
/// Fails silently with a warning — this is best-effort provenance metadata.
async fn set_round_applied_patterns(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    round_id: Uuid,
    patterns: &[DecisionPatternResponse],
) {
    if patterns.is_empty() {
        return;
    }
    let summaries: Vec<AppliedPatternSummary> = patterns
        .iter()
        .map(|p| AppliedPatternSummary {
            id: p.id,
            domain: p.domain.clone(),
            pattern: p.pattern.clone(),
            confidence: p.confidence,
            override_count: p.override_count,
        })
        .collect();

    let mut tx = match OrgTx::begin(pool, org_id).await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::warn!(%round_id, %e, "failed to begin tx for applied_patterns update");
            return;
        }
    };
    let row = match qa_repo::get_round(&mut tx, round_id, org_id).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            tracing::warn!(%round_id, "round not found when recording applied patterns");
            let _ = tx.commit().await;
            return;
        }
        Err(e) => {
            tracing::warn!(%round_id, %e, "failed to fetch round for applied_patterns update");
            return;
        }
    };
    let mut content: QaContent = match serde_json::from_value(row.content) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(%round_id, %e, "failed to parse round content for applied_patterns update");
            return;
        }
    };
    content.applied_patterns = summaries;
    let content_json = match serde_json::to_value(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(%round_id, %e, "failed to serialize content for applied_patterns update");
            return;
        }
    };
    if let Err(e) = qa_repo::update_round_content(&mut tx, round_id, org_id, &content_json).await {
        tracing::warn!(%round_id, %e, "failed to write applied_patterns to round");
        return;
    }
    if let Err(e) = tx.commit().await {
        tracing::warn!(%round_id, %e, "failed to commit applied_patterns update");
    } else {
        tracing::info!(%round_id, count = patterns.len(), "recorded applied patterns for Q&A round");
    }
}

/// Pre-create an empty shared grooming round that all parallel roles will
/// append their questions into.  Returns `(round_id, round_number)`.
async fn create_shared_grooming_round(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<(Uuid, i32), ApiError> {
    let mut tx = OrgTx::begin(pool, org_id).await?;

    let max_round = qa_repo::get_max_round_number(&mut tx, story_id, None, "grooming")
        .await?
        .unwrap_or(0);
    let round_number = max_round + 1;

    let empty_content = serde_json::json!({ "questions": [], "course_correction": null });
    let row = qa_repo::create_round(
        &mut tx,
        org_id,
        story_id,
        None,
        "grooming",
        round_number,
        &empty_content,
    )
    .await?;

    tx.commit().await?;
    Ok((row.id, round_number))
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
    /// Patterns piggybacked on the QA output (0-3 per round).
    #[serde(default)]
    patterns: Vec<ExtractedPatternOutput>,
}

/// Pattern extracted from LLM output, piggybacked on QA round.
#[derive(serde::Deserialize)]
struct ExtractedPatternOutput {
    domain: String,
    pattern: String,
    rationale: String,
    #[serde(default)]
    tags: Vec<String>,
}

impl From<ExtractedPatternOutput> for ExtractedPattern {
    fn from(p: ExtractedPatternOutput) -> Self {
        ExtractedPattern {
            domain: p.domain,
            pattern: p.pattern,
            rationale: p.rationale,
            tags: p.tags,
        }
    }
}

/// Output format for roles 1-N in the sequential grooming chain.
#[derive(serde::Deserialize)]
struct SequentialQaRoundOutput {
    #[serde(default)]
    augmentations: Vec<QaAugmentationOutput>,
    #[serde(default)]
    questions: Vec<QaQuestionOutput>,
    #[serde(default)]
    patterns: Vec<ExtractedPatternOutput>,
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
