use std::ops::{Deref, DerefMut};

use axum::{extract::FromRequestParts, http::request::Parts};
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::{auth::types::AuthContext, errors::ApiError, state::AppState};

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to database: {e}"))
}

/// Generates a new UUIDv7 identifier (time-ordered, application-side).
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

/// A transaction with the RLS `app.org_id` context already set.
///
/// Implements `Deref`/`DerefMut` to the inner transaction so repo functions
/// accepting `&mut Transaction<'_, Postgres>` work unchanged.
pub struct OrgTx {
    tx: Transaction<'static, Postgres>,
    pub org_id: Uuid,
}

impl OrgTx {
    /// Begin a transaction and set `app.org_id` for RLS.
    pub async fn begin(pool: &PgPool, org_id: Uuid) -> Result<Self, ApiError> {
        let mut tx = pool.begin().await?;
        sqlx::query("SELECT set_config('app.org_id', $1, true)")
            .bind(org_id.to_string())
            .execute(&mut *tx)
            .await?;
        Ok(Self { tx, org_id })
    }

    /// Commit the underlying transaction.
    pub async fn commit(self) -> Result<(), ApiError> {
        self.tx.commit().await?;
        Ok(())
    }
}

impl Deref for OrgTx {
    type Target = Transaction<'static, Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl DerefMut for OrgTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}

/// Axum extractor: begins a transaction with RLS context from the authenticated user.
///
/// Requires `require_auth` middleware to have inserted `AuthContext` into extensions.
impl FromRequestParts<AppState> for OrgTx {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or(ApiError::Unauthorized)?;
        Self::begin(&state.pool, auth.org_id).await
    }
}
