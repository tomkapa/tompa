use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::db::new_id;

#[derive(Debug, sqlx::FromRow)]
pub struct DecisionPatternRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Option<Uuid>,
    pub domain: String,
    pub pattern: String,
    pub rationale: String,
    pub tags: Vec<String>,
    pub confidence: f32,
    pub usage_count: i32,
    pub override_count: i32,
    pub source_story_id: Option<Uuid>,
    pub source_round_id: Option<Uuid>,
    pub superseded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result from trigram similarity search for dedup.
#[derive(Debug, sqlx::FromRow)]
pub struct SimilarPattern {
    pub id: Uuid,
    pub pattern: String,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub sim: f32,
}

/// Insert a new decision pattern. Returns the created row.
pub async fn insert_pattern(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Option<Uuid>,
    domain: &str,
    pattern: &str,
    rationale: &str,
    tags: &[String],
    confidence: f32,
    source_story_id: Option<Uuid>,
    source_round_id: Option<Uuid>,
) -> Result<DecisionPatternRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        INSERT INTO decision_patterns
            (id, org_id, project_id, domain, pattern, rationale, tags, confidence,
             source_story_id, source_round_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, org_id, project_id, domain, pattern, rationale, tags, confidence,
                  usage_count, override_count, source_story_id, source_round_id,
                  superseded_by, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind(domain)
    .bind(pattern)
    .bind(rationale)
    .bind(tags)
    .bind(confidence)
    .bind(source_story_id)
    .bind(source_round_id)
    .fetch_one(pool)
    .await
}

/// Find similar patterns using pg_trgm similarity.
pub async fn find_similar_patterns(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Option<Uuid>,
    domain: &str,
    pattern_text: &str,
) -> Result<Vec<SimilarPattern>, sqlx::Error> {
    sqlx::query_as::<_, SimilarPattern>(
        r#"
        SELECT id, pattern, confidence, tags,
               similarity(pattern, $1) AS sim
        FROM decision_patterns
        WHERE org_id = $2
          AND (project_id IS NULL OR project_id = $3)
          AND domain = $4
          AND superseded_by IS NULL
          AND deleted_at IS NULL
          AND similarity(pattern, $1) > 0.4
        ORDER BY sim DESC
        LIMIT 5
        "#,
    )
    .bind(pattern_text)
    .bind(org_id)
    .bind(project_id)
    .bind(domain)
    .fetch_all(pool)
    .await
}

/// Bump confidence of an existing pattern (reinforcement).
pub async fn reinforce_pattern(pool: &PgPool, pattern_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE decision_patterns
        SET confidence = LEAST(confidence + 0.05, 1.0),
            usage_count = usage_count + 1,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(pattern_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Increment usage_count for a pattern.
pub async fn increment_usage(pool: &PgPool, pattern_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE decision_patterns
        SET usage_count = usage_count + 1,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(pattern_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Increment override_count for a pattern.
pub async fn increment_override(pool: &PgPool, pattern_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE decision_patterns
        SET override_count = override_count + 1,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(pattern_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Recalculate confidence based on usage/override counts.
/// Formula: base_confidence * (usage_count / (usage_count + override_count * 2))
pub async fn recalculate_confidence(pool: &PgPool, pattern_id: Uuid) -> Result<f32, sqlx::Error> {
    let row: (f32,) = sqlx::query_as(
        r#"
        UPDATE decision_patterns
        SET confidence = CASE
                WHEN usage_count + override_count = 0 THEN 0.8
                ELSE 0.8 * (usage_count::real / (usage_count + override_count * 2)::real)
            END,
            updated_at = now()
        WHERE id = $1
        RETURNING confidence
        "#,
    )
    .bind(pattern_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Auto-archive patterns with confidence < 0.3 (set deleted_at).
pub async fn auto_archive_low_confidence(
    pool: &PgPool,
    pattern_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE decision_patterns
        SET deleted_at = now(), updated_at = now()
        WHERE id = $1
          AND confidence < 0.3
          AND deleted_at IS NULL
        "#,
    )
    .bind(pattern_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Fetch relevant patterns using FTS + tag overlap for prompt injection.
/// `exclude_story_id` prevents patterns extracted from the current story from being
/// fed back into subsequent rounds of the same session (circular self-reference).
pub async fn fetch_relevant_patterns(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
    search_text: &str,
    tags: &[String],
    exclude_story_id: Option<Uuid>,
) -> Result<Vec<DecisionPatternRow>, sqlx::Error> {
    sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        SELECT id, org_id, project_id, domain, pattern, rationale, tags, confidence,
               usage_count, override_count, source_story_id, source_round_id,
               superseded_by, created_at, updated_at
        FROM decision_patterns
        WHERE org_id = $1
          AND (project_id IS NULL OR project_id = $2)
          AND superseded_by IS NULL
          AND deleted_at IS NULL
          AND confidence > 0.5
          AND ($5::uuid IS NULL OR source_story_id IS NULL OR source_story_id != $5)
          AND (
            search_vector @@ websearch_to_tsquery('english', $3)
            OR tags && $4::text[]
          )
        ORDER BY ts_rank(search_vector, websearch_to_tsquery('english', $3)) * confidence DESC
        LIMIT 10
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .bind(search_text)
    .bind(tags)
    .bind(exclude_story_id)
    .fetch_all(pool)
    .await
}

/// List all active patterns for a project, with optional filters.
pub async fn list_patterns(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Uuid,
    domain: Option<&str>,
    min_confidence: Option<f32>,
) -> Result<Vec<DecisionPatternRow>, sqlx::Error> {
    sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        SELECT id, org_id, project_id, domain, pattern, rationale, tags, confidence,
               usage_count, override_count, source_story_id, source_round_id,
               superseded_by, created_at, updated_at
        FROM decision_patterns
        WHERE org_id = $1
          AND project_id = $2
          AND deleted_at IS NULL
          AND superseded_by IS NULL
          AND ($3::text IS NULL OR domain = $3)
          AND confidence >= COALESCE($4, 0.0)
        ORDER BY confidence DESC, created_at DESC
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .bind(domain)
    .bind(min_confidence)
    .fetch_all(&mut **tx)
    .await
}

/// Get a single pattern by ID.
pub async fn get_pattern(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
) -> Result<Option<DecisionPatternRow>, sqlx::Error> {
    sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        SELECT id, org_id, project_id, domain, pattern, rationale, tags, confidence,
               usage_count, override_count, source_story_id, source_round_id,
               superseded_by, created_at, updated_at
        FROM decision_patterns
        WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Update a pattern's text, rationale, or tags.
pub async fn update_pattern(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
    pattern: Option<&str>,
    rationale: Option<&str>,
    tags: Option<&[String]>,
) -> Result<Option<DecisionPatternRow>, sqlx::Error> {
    sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        UPDATE decision_patterns
        SET pattern   = COALESCE($3, pattern),
            rationale = COALESCE($4, rationale),
            tags      = COALESCE($5, tags),
            updated_at = now()
        WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
        RETURNING id, org_id, project_id, domain, pattern, rationale, tags, confidence,
                  usage_count, override_count, source_story_id, source_round_id,
                  superseded_by, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(pattern)
    .bind(rationale)
    .bind(tags)
    .fetch_optional(&mut **tx)
    .await
}

/// Soft-delete (retire) a pattern.
pub async fn retire_pattern(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE decision_patterns
        SET deleted_at = now(), updated_at = now()
        WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(org_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Supersede a pattern: mark old as superseded, insert replacement.
pub async fn supersede_pattern(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    old_id: Uuid,
    org_id: Uuid,
    new_pattern: &str,
    new_rationale: &str,
    new_tags: &[String],
) -> Result<Option<DecisionPatternRow>, sqlx::Error> {
    let new_id = new_id();

    // Get the old pattern for domain/project_id
    let old = sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        SELECT id, org_id, project_id, domain, pattern, rationale, tags, confidence,
               usage_count, override_count, source_story_id, source_round_id,
               superseded_by, created_at, updated_at
        FROM decision_patterns
        WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL AND superseded_by IS NULL
        "#,
    )
    .bind(old_id)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await?;

    let Some(old) = old else {
        return Ok(None);
    };

    // Insert the new replacement pattern
    let new_row = sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        INSERT INTO decision_patterns
            (id, org_id, project_id, domain, pattern, rationale, tags, confidence,
             source_story_id, source_round_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0.8, $8, $9)
        RETURNING id, org_id, project_id, domain, pattern, rationale, tags, confidence,
                  usage_count, override_count, source_story_id, source_round_id,
                  superseded_by, created_at, updated_at
        "#,
    )
    .bind(new_id)
    .bind(org_id)
    .bind(old.project_id)
    .bind(&old.domain)
    .bind(new_pattern)
    .bind(new_rationale)
    .bind(new_tags)
    .bind(old.source_story_id)
    .bind(old.source_round_id)
    .fetch_one(&mut **tx)
    .await?;

    // Mark the old pattern as superseded
    sqlx::query(
        r#"
        UPDATE decision_patterns
        SET superseded_by = $3, updated_at = now()
        WHERE id = $1 AND org_id = $2
        "#,
    )
    .bind(old_id)
    .bind(org_id)
    .bind(new_id)
    .execute(&mut **tx)
    .await?;

    Ok(Some(new_row))
}

/// Fetch active patterns for a project (used for feedback alignment check).
pub async fn fetch_active_patterns_for_project(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<Vec<DecisionPatternRow>, sqlx::Error> {
    sqlx::query_as::<_, DecisionPatternRow>(
        r#"
        SELECT id, org_id, project_id, domain, pattern, rationale, tags, confidence,
               usage_count, override_count, source_story_id, source_round_id,
               superseded_by, created_at, updated_at
        FROM decision_patterns
        WHERE org_id = $1
          AND (project_id IS NULL OR project_id = $2)
          AND superseded_by IS NULL
          AND deleted_at IS NULL
          AND confidence > 0.3
        ORDER BY confidence DESC
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Count new and total active patterns for threshold check.
pub async fn count_patterns_for_threshold(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<(i64, i64), sqlx::Error> {
    let row: (i64, i64) = sqlx::query_as(
        r#"
        WITH profile_baseline AS (
            SELECT COALESCE(
                (SELECT generated_at FROM project_profiles
                 WHERE project_id = $1 AND org_id = $2),
                '1970-01-01'::timestamptz
            ) AS last_generated
        )
        SELECT
            (SELECT COUNT(*) FROM decision_patterns
             WHERE project_id = $1 AND org_id = $2
               AND superseded_by IS NULL AND deleted_at IS NULL
               AND created_at > (SELECT last_generated FROM profile_baseline)
            ) AS new_count,
            (SELECT COUNT(*) FROM decision_patterns
             WHERE project_id = $1 AND org_id = $2
               AND superseded_by IS NULL AND deleted_at IS NULL
            ) AS total_count
        "#,
    )
    .bind(project_id)
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}
