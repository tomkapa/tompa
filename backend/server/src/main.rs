use std::sync::Arc;

use server::{
    agents::registry::DashMapRegistry, build_app, config::Config, db,
    sse::broadcaster::SseBroadcaster, state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::telemetry::init_tracing("server");

    let config = Config::from_env();
    let port = config.port;

    let pool = db::create_pool(&config.database_url).await;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState {
        pool,
        config: Arc::new(config),
        registry: Arc::new(DashMapRegistry::new()),
        broadcaster: Arc::new(SseBroadcaster::new()),
    };

    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!(port, "Server listening");
    axum::serve(listener, app).await?;

    Ok(())
}
