use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::OrgTx, errors::ApiError};

use super::{
    repo,
    types::{
        DecisionPatternError, DecisionPatternResponse, ExtractedPattern, PatternClassification,
        SupersedePatternRequest, UpdatePatternRequest, is_valid_domain,
    },
};

// ── Row → Response conversion ────────────────────────────────────────────────

fn to_response(row: repo::DecisionPatternRow) -> DecisionPatternResponse {
    DecisionPatternResponse {
        id: row.id,
        org_id: row.org_id,
        project_id: row.project_id,
        domain: row.domain,
        pattern: row.pattern,
        rationale: row.rationale,
        tags: row.tags,
        confidence: row.confidence,
        usage_count: row.usage_count,
        override_count: row.override_count,
        source_story_id: row.source_story_id,
        source_round_id: row.source_round_id,
        superseded_by: row.superseded_by,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

// ── Dedup classification logic (pg_trgm + Jaccard, no LLM) ──────────────────

/// Classify a new pattern against existing patterns in the DB.
/// Uses trigram similarity + Jaccard tag overlap.
pub async fn classify_pattern(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Option<Uuid>,
    new_pattern: &ExtractedPattern,
) -> Result<PatternClassification, ApiError> {
    let similar = repo::find_similar_patterns(
        pool,
        org_id,
        project_id,
        &new_pattern.domain,
        &new_pattern.pattern,
    )
    .await?;

    if similar.is_empty() {
        return Ok(PatternClassification::New);
    }

    let best = &similar[0];

    if best.sim > 0.8 {
        return Ok(PatternClassification::Duplicate {
            existing_id: best.id,
        });
    }

    if best.sim > 0.5 {
        let tag_jaccard = jaccard_similarity(&new_pattern.tags, &best.tags);
        if tag_jaccard > 0.5 {
            // Check for contradictory signal via opposing tags
            if has_contradictory_tags(&new_pattern.tags, &best.tags) {
                return Ok(PatternClassification::Contradicts {
                    existing_id: best.id,
                });
            }
            return Ok(PatternClassification::Reinforces {
                existing_id: best.id,
            });
        }
    }

    // sim 0.4–0.5 or no tag overlap — treat as new
    Ok(PatternClassification::New)
}

/// Store a pattern based on its classification.
pub async fn store_pattern(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Option<Uuid>,
    story_id: Option<Uuid>,
    round_id: Option<Uuid>,
    pattern: &ExtractedPattern,
    classification: &PatternClassification,
) -> Result<(), ApiError> {
    match classification {
        PatternClassification::Duplicate { existing_id } => {
            tracing::debug!(
                %org_id,
                existing_pattern_id = %existing_id,
                "pattern classified as DUPLICATE — skipping insert"
            );
        }
        PatternClassification::Reinforces { existing_id } => {
            tracing::info!(
                %org_id,
                existing_pattern_id = %existing_id,
                "pattern classified as REINFORCES — bumping confidence"
            );
            repo::reinforce_pattern(pool, *existing_id).await?;
        }
        PatternClassification::Contradicts { existing_id } => {
            tracing::warn!(
                %org_id,
                existing_pattern_id = %existing_id,
                pattern = %pattern.pattern,
                "pattern classified as CONTRADICTS — inserting with flag"
            );
            repo::insert_pattern(
                pool,
                org_id,
                project_id,
                &pattern.domain,
                &pattern.pattern,
                &pattern.rationale,
                &pattern.tags,
                0.8,
                story_id,
                round_id,
            )
            .await?;
        }
        PatternClassification::New => {
            tracing::info!(
                %org_id,
                domain = %pattern.domain,
                "pattern classified as NEW — inserting"
            );
            repo::insert_pattern(
                pool,
                org_id,
                project_id,
                &pattern.domain,
                &pattern.pattern,
                &pattern.rationale,
                &pattern.tags,
                0.8,
                story_id,
                round_id,
            )
            .await?;
        }
    }
    Ok(())
}

/// Process extracted patterns from a QA round output.
pub async fn process_extracted_patterns(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Uuid,
    round_id: Option<Uuid>,
    patterns: Vec<ExtractedPattern>,
) {
    for pattern in &patterns {
        if !is_valid_domain(&pattern.domain) {
            tracing::warn!(
                %org_id,
                %project_id,
                %story_id,
                domain = %pattern.domain,
                "extracted pattern has invalid domain — skipping"
            );
            continue;
        }

        let classification = match classify_pattern(pool, org_id, Some(project_id), pattern).await
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(
                    %org_id,
                    %project_id,
                    %story_id,
                    %e,
                    "failed to classify pattern — skipping"
                );
                continue;
            }
        };

        tracing::info!(
            %org_id,
            %project_id,
            %story_id,
            domain = %pattern.domain,
            classification = ?classification,
            "pattern classification result"
        );

        if let Err(e) = store_pattern(
            pool,
            org_id,
            Some(project_id),
            Some(story_id),
            round_id,
            pattern,
            &classification,
        )
        .await
        {
            tracing::error!(
                %org_id,
                %project_id,
                %story_id,
                %e,
                "failed to store pattern"
            );
        }
    }
}

// ── Jaccard similarity helpers ───────────────────────────────────────────────

fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let a_set: std::collections::HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let b_set: std::collections::HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let intersection = a_set.intersection(&b_set).count();
    let union = a_set.union(&b_set).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Simple heuristic: check if tags contain opposing prefixes like "sync" vs "async",
/// "rest" vs "graphql", etc.
fn has_contradictory_tags(new_tags: &[String], existing_tags: &[String]) -> bool {
    let contradictions: &[(&str, &str)] = &[
        ("sync", "async"),
        ("rest", "graphql"),
        ("monolith", "microservice"),
        ("sql", "nosql"),
        ("ssr", "spa"),
        ("mutable", "immutable"),
    ];

    let new_lower: Vec<String> = new_tags.iter().map(|t| t.to_lowercase()).collect();
    let existing_lower: Vec<String> = existing_tags.iter().map(|t| t.to_lowercase()).collect();

    for (a, b) in contradictions {
        let new_has_a = new_lower.iter().any(|t| t.contains(a));
        let new_has_b = new_lower.iter().any(|t| t.contains(b));
        let existing_has_a = existing_lower.iter().any(|t| t.contains(a));
        let existing_has_b = existing_lower.iter().any(|t| t.contains(b));

        if (new_has_a && existing_has_b) || (new_has_b && existing_has_a) {
            return true;
        }
    }
    false
}

// ── Feedback loop: confidence evolution ──────────────────────────────────────

/// Update pattern confidence after answer alignment check.
/// Returns the new confidence and whether the pattern was auto-archived.
pub async fn update_pattern_feedback(
    pool: &PgPool,
    pattern_id: Uuid,
    aligned: bool,
) -> Result<(f32, bool), ApiError> {
    if aligned {
        repo::increment_usage(pool, pattern_id).await?;
    } else {
        repo::increment_override(pool, pattern_id).await?;
    }

    let new_confidence = repo::recalculate_confidence(pool, pattern_id).await?;

    let archived = if new_confidence < 0.3 {
        repo::auto_archive_low_confidence(pool, pattern_id).await?
    } else {
        false
    };

    if archived {
        tracing::info!(
            %pattern_id,
            new_confidence,
            "pattern auto-archived due to low confidence"
        );
    }

    Ok((new_confidence, archived))
}

// ── Feedback loop: process answer alignment ──────────────────────────────────

/// Process pattern feedback after all answers in a round are submitted.
/// Re-fetches relevant patterns for the project, compares answer texts,
/// and updates usage_count / override_count accordingly.
pub async fn process_answer_feedback(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
    answers: &[(String, String)], // (question_text, answer_text)
) {
    let patterns = match repo::fetch_active_patterns_for_project(pool, org_id, project_id).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(
                %org_id, %project_id, %e,
                "failed to fetch patterns for feedback loop"
            );
            return;
        }
    };

    if patterns.is_empty() {
        return;
    }

    // Combine all answer text for keyword matching
    let combined_answers: String = answers
        .iter()
        .map(|(q, a)| format!("{} {}", q, a))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    for pattern in &patterns {
        let pattern_lower = pattern.pattern.to_lowercase();
        let pattern_words: Vec<&str> = pattern_lower
            .split_whitespace()
            .filter(|w| w.len() > 3) // skip short words
            .collect();

        // Check if at least 30% of pattern keywords appear in answers
        if pattern_words.is_empty() {
            continue;
        }

        let matching_words = pattern_words
            .iter()
            .filter(|w| combined_answers.contains(**w))
            .count();

        let relevance = matching_words as f64 / pattern_words.len() as f64;

        // Only process patterns that seem relevant to this round's questions
        if relevance < 0.3 {
            continue;
        }

        // Check for contradiction signals in answer text
        let contradicted = answer_contradicts_pattern(&combined_answers, &pattern_lower);

        let (new_confidence, archived) =
            match update_pattern_feedback(pool, pattern.id, !contradicted).await {
                Ok(result) => result,
                Err(e) => {
                    tracing::error!(
                        %org_id,
                        pattern_id = %pattern.id,
                        %e,
                        "failed to update pattern feedback"
                    );
                    continue;
                }
            };

        tracing::info!(
            %org_id,
            %project_id,
            pattern_id = %pattern.id,
            aligned = !contradicted,
            new_confidence,
            archived,
            "pattern feedback processed"
        );

        // Flag patterns with high override count
        if pattern.override_count + if contradicted { 1 } else { 0 } > 3 {
            tracing::warn!(
                %org_id,
                %project_id,
                pattern_id = %pattern.id,
                override_count = pattern.override_count + if contradicted { 1 } else { 0 },
                pattern = %pattern.pattern,
                "pattern has high override count — may need review or superseding"
            );
        }
    }
}

/// Simple heuristic to detect if answer text contradicts a pattern.
/// Looks for negation signals near pattern keywords.
fn answer_contradicts_pattern(answer_text: &str, pattern_text: &str) -> bool {
    let negation_phrases = [
        "instead of",
        "rather than",
        "not use",
        "don't use",
        "won't use",
        "avoid",
        "different approach",
        "different from",
        "contrary to",
        "override",
        "disagree",
        "reject",
        "no longer",
        "deprecated",
        "moved away from",
        "switched from",
    ];

    // Extract key concept words from pattern (>4 chars to avoid noise)
    let concept_words: Vec<&str> = pattern_text
        .split_whitespace()
        .filter(|w| w.len() > 4)
        .collect();

    if concept_words.is_empty() {
        return false;
    }

    // Check if any negation phrase appears near pattern concepts in the answer
    for phrase in &negation_phrases {
        if let Some(neg_pos) = answer_text.find(phrase) {
            // Check if any concept word appears within 100 chars of the negation
            let window_start = neg_pos.saturating_sub(100);
            let window_end = (neg_pos + phrase.len() + 100).min(answer_text.len());
            let window = &answer_text[window_start..window_end];

            let concepts_near = concept_words
                .iter()
                .any(|concept| window.contains(concept));

            if concepts_near {
                return true;
            }
        }
    }

    false
}

// ── CRUD service functions (for REST API) ────────────────────────────────────

pub async fn list_patterns(
    tx: &mut OrgTx,
    project_id: Uuid,
    domain: Option<&str>,
    min_confidence: Option<f32>,
) -> Result<Vec<DecisionPatternResponse>, ApiError> {
    let org_id = tx.org_id;
    let rows = repo::list_patterns(tx, org_id, project_id, domain, min_confidence).await?;
    Ok(rows.into_iter().map(to_response).collect())
}

pub async fn get_pattern(
    tx: &mut OrgTx,
    pattern_id: Uuid,
) -> Result<DecisionPatternResponse, ApiError> {
    let org_id = tx.org_id;
    let row = repo::get_pattern(tx, pattern_id, org_id)
        .await?
        .ok_or(DecisionPatternError::NotFound)?;
    Ok(to_response(row))
}

pub async fn update_pattern(
    tx: &mut OrgTx,
    pattern_id: Uuid,
    req: UpdatePatternRequest,
) -> Result<DecisionPatternResponse, ApiError> {
    if let Some(ref p) = req.pattern {
        if p.trim().is_empty() {
            return Err(DecisionPatternError::PatternRequired.into());
        }
    }
    if let Some(ref r) = req.rationale {
        if r.trim().is_empty() {
            return Err(DecisionPatternError::RationaleRequired.into());
        }
    }

    let org_id = tx.org_id;
    let row = repo::update_pattern(
        tx,
        pattern_id,
        org_id,
        req.pattern.as_deref(),
        req.rationale.as_deref(),
        req.tags.as_deref(),
    )
    .await?
    .ok_or(DecisionPatternError::NotFound)?;
    Ok(to_response(row))
}

pub async fn retire_pattern(tx: &mut OrgTx, pattern_id: Uuid) -> Result<(), ApiError> {
    let org_id = tx.org_id;
    let retired = repo::retire_pattern(tx, pattern_id, org_id).await?;
    if !retired {
        return Err(DecisionPatternError::NotFound.into());
    }
    Ok(())
}

pub async fn supersede_pattern(
    tx: &mut OrgTx,
    pattern_id: Uuid,
    req: SupersedePatternRequest,
) -> Result<DecisionPatternResponse, ApiError> {
    if req.pattern.trim().is_empty() {
        return Err(DecisionPatternError::SupersedePatternRequired.into());
    }
    if req.rationale.trim().is_empty() {
        return Err(DecisionPatternError::RationaleRequired.into());
    }

    let org_id = tx.org_id;
    let tags = req.tags.unwrap_or_default();
    let row = repo::supersede_pattern(tx, pattern_id, org_id, &req.pattern, &req.rationale, &tags)
        .await?
        .ok_or(DecisionPatternError::NotFound)?;
    Ok(to_response(row))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaccard_identical_tags() {
        let a = vec!["rest".to_string(), "api".to_string()];
        let b = vec!["rest".to_string(), "api".to_string()];
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_no_overlap() {
        let a = vec!["rest".to_string()];
        let b = vec!["graphql".to_string()];
        assert!((jaccard_similarity(&a, &b)).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let a = vec!["rest".to_string(), "api".to_string(), "json".to_string()];
        let b = vec!["rest".to_string(), "xml".to_string()];
        // intersection = {rest} = 1, union = {rest, api, json, xml} = 4
        assert!((jaccard_similarity(&a, &b) - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_both_empty() {
        let a: Vec<String> = vec![];
        let b: Vec<String> = vec![];
        assert!((jaccard_similarity(&a, &b)).abs() < f64::EPSILON);
    }

    #[test]
    fn contradictory_tags_detected() {
        let new = vec!["sync".to_string(), "api".to_string()];
        let existing = vec!["async".to_string(), "api".to_string()];
        assert!(has_contradictory_tags(&new, &existing));
    }

    #[test]
    fn non_contradictory_tags() {
        let new = vec!["rest".to_string(), "api".to_string()];
        let existing = vec!["rest".to_string(), "json".to_string()];
        assert!(!has_contradictory_tags(&new, &existing));
    }

    #[test]
    fn contradictory_rest_graphql() {
        let new = vec!["rest".to_string()];
        let existing = vec!["graphql".to_string()];
        assert!(has_contradictory_tags(&new, &existing));
    }

    #[test]
    fn answer_contradicts_with_negation_near_concept() {
        let answer = "we decided to avoid using async processing for this pipeline";
        let pattern = "prefer async processing for data pipelines";
        assert!(answer_contradicts_pattern(answer, pattern));
    }

    #[test]
    fn answer_does_not_contradict_when_aligned() {
        let answer = "we should use async processing for the data pipeline as usual";
        let pattern = "prefer async processing for data pipelines";
        assert!(!answer_contradicts_pattern(answer, pattern));
    }

    #[test]
    fn answer_contradicts_with_instead_of() {
        let answer =
            "for the public endpoints we chose graphql instead of the standard approach";
        let pattern = "use rest apis for all public endpoints";
        assert!(answer_contradicts_pattern(answer, pattern));
    }

    #[test]
    fn answer_no_contradiction_unrelated() {
        let answer = "the color scheme should use dark mode by default";
        let pattern = "prefer async processing for data pipelines";
        assert!(!answer_contradicts_pattern(answer, pattern));
    }
}
