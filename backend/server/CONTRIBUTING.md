# Repository Pattern

Every domain repository function follows this pattern to ensure correct
multi-tenant isolation via Row-Level Security (RLS).

## The Belt-and-Suspenders Rule

- **Belt:** every `SELECT` / `UPDATE` / `DELETE` query includes an explicit
  `AND org_id = $N` predicate.
- **Suspenders:** RLS is enabled on every tenant-scoped table
  (`012_enable_rls.sql`) and enforced via `app.org_id` set for the current
  transaction before any query runs.

Both layers must be present. Belt keeps queries fast (uses the index); RLS is
the backstop that prevents data leaks if a bug forgets the belt.

## Function Signature

```rust
pub async fn find_by_id(
    pool: &PgPool,
    org_id: Uuid,
    id: Uuid,
) -> Result<Option<Story>, sqlx::Error>
```

- `pool: &PgPool` — shared connection pool, injected from `AppState`
- `org_id: Uuid` — extracted from the verified JWT by the auth middleware;
  passed as an explicit parameter to every repo call

## Setting the RLS Context

Middleware acquires a connection, opens a transaction, and calls
`db::set_rls_context` before passing control to the handler:

```rust
// In middleware / handler setup:
let mut tx = pool.begin().await?;
db::set_rls_context(&mut *tx, org_id).await?;

// Repository functions receive &mut tx and run inside the same transaction.
// When the transaction commits or rolls back, SET LOCAL is automatically
// cleared — the pooled connection is returned clean.
```

## Query Rules

1. **`AND org_id = $N`** in every `WHERE` clause (SELECT, UPDATE, DELETE).
2. **`deleted_at IS NULL`** in every `SELECT` query (soft-delete pattern).
3. **All IDs generated application-side** via `db::new_id()` (`Uuid::now_v7()`).
   Never rely on the database to generate primary keys.

## Example Repository Function

```rust
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

struct StoryRow {
    id: Uuid,
    org_id: Uuid,
    project_id: Uuid,
    title: String,
    description: String,
    story_type: String,
    status: String,
    owner_id: Uuid,
    rank: String,
    pipeline_stage: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

pub async fn find_by_id(
    pool: &PgPool,
    org_id: Uuid,
    id: Uuid,
) -> Result<Option<StoryRow>, sqlx::Error> {
    sqlx::query_as!(
        StoryRow,
        r#"SELECT id, org_id, project_id, title, description,
                  story_type, status, owner_id, rank, pipeline_stage,
                  created_at, updated_at, deleted_at
           FROM stories
           WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL"#,
        id,
        org_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
    title: String,
) -> Result<StoryRow, sqlx::Error> {
    let id = crate::db::new_id(); // UUIDv7, generated application-side

    sqlx::query_as!(
        StoryRow,
        r#"INSERT INTO stories (id, org_id, project_id, title, ...)
           VALUES ($1, $2, $3, $4, ...)
           RETURNING id, org_id, project_id, title, description,
                     story_type, status, owner_id, rank, pipeline_stage,
                     created_at, updated_at, deleted_at"#,
        id,
        org_id,
        project_id,
        title
    )
    .fetch_one(pool)
    .await
}
```

## Soft Deletes

Never `DELETE` rows. Set `deleted_at = now()` instead:

```rust
sqlx::query!(
    "UPDATE stories SET deleted_at = now() WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL",
    id,
    org_id
)
.execute(pool)
.await?;
```
