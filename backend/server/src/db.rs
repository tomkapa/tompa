use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to database: {e}"))
}

/// Sets `app.org_id` for RLS on the current connection.
///
/// **Must be called within an open transaction** so that `SET LOCAL` scopes
/// the setting to that transaction only — it is reset automatically on
/// COMMIT/ROLLBACK, keeping pooled connections clean.
///
/// The middleware is responsible for starting the transaction, calling this
/// function, and then passing `&mut tx` down to all repository functions.
pub async fn set_rls_context(pool: &PgPool, org_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(&format!("SET LOCAL app.org_id = '{org_id}'"))
        .execute(pool)
        .await?;
    Ok(())
}

/// Generates a new UUIDv7 identifier (time-ordered, application-side).
///
/// All primary keys are generated here rather than in the database so that
/// the ID is known before the INSERT, making it easy to return from handlers
/// without a second round-trip.
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}
