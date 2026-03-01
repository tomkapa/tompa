use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    agents::registry::ConnectionRegistry, config::Config, sse::broadcaster::SseBroadcaster,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub registry: Arc<dyn ConnectionRegistry>,
    pub broadcaster: Arc<SseBroadcaster>,
}
