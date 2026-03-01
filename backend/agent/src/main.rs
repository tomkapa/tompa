mod agent_status;
mod claude_code;
mod config;
mod dispatcher;
mod git_manager;
mod prompts;
mod setup_ui;
mod ws_client;

use anyhow::Result;
use tokio::sync::{mpsc, watch};
use tracing::info;

use shared::enums::ContainerMode;

use crate::{
    agent_status::ConnectionStatus,
    claude_code::ClaudeCode,
    config::Config,
    dispatcher::{DispatchMessage, Dispatcher},
    git_manager::GitManager,
    setup_ui::SetupUi,
    ws_client::WsClient,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::load().map_err(|e| {
        eprintln!("ERROR: {e}");
        e
    })?;

    info!(mode = ?config.mode, "Agent starting");

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

    let (dispatch_tx, dispatch_rx) = mpsc::channel::<DispatchMessage>(256);

    // Shared connection status — written by WsClient, read by SetupUi HTTP handler.
    let (status_tx, status_rx) = watch::channel(ConnectionStatus::default());

    // WebSocket actor — present in all modes
    let (ws_tx, ws_rx) = mpsc::channel(64);
    let ws_actor = WsClient::new(
        config.server_url.clone(),
        config.api_key.clone(),
        dispatch_tx.clone(),
        ws_rx,
        status_tx,
    );

    // Claude Code actor — present in all modes (handles both Q&A and implementation)
    let (claude_tx, claude_rx) = mpsc::channel(64);
    let claude_actor = ClaudeCode::new(dispatch_tx.clone(), claude_rx);

    // Git manager — Dev and Standalone only
    let (git_tx, git_actor) = match config.mode {
        ContainerMode::Dev | ContainerMode::Standalone => {
            let (tx, rx) = mpsc::channel(64);
            let actor = GitManager::new(
                config.github_repo_url.clone(),
                config.github_access_token.clone(),
                dispatch_tx.clone(),
                rx,
            );
            (Some(tx), Some(actor))
        }
        ContainerMode::Project => (None, None),
    };

    // Setup UI — Project and Standalone only
    let (ui_tx, ui_actor) = match config.mode {
        ContainerMode::Project | ContainerMode::Standalone => {
            let port = config.setup_ui_port.unwrap_or(3001);
            let mode_str = match config.mode {
                ContainerMode::Project => "project",
                ContainerMode::Dev => "dev",
                ContainerMode::Standalone => "standalone",
            }
            .to_string();
            let (tx, rx) = mpsc::channel(16);
            let actor = SetupUi::new(
                port,
                config_path,
                mode_str,
                dispatch_tx.clone(),
                rx,
                status_rx,
            );
            (Some(tx), Some(actor))
        }
        ContainerMode::Dev => (None, None),
    };

    let dispatcher = Dispatcher::new(dispatch_rx, Some(ws_tx), Some(claude_tx), git_tx, ui_tx);

    // Spawn all active actors
    tokio::spawn(ws_actor.run());
    tokio::spawn(claude_actor.run());
    if let Some(actor) = git_actor {
        tokio::spawn(actor.run());
    }
    if let Some(actor) = ui_actor {
        tokio::spawn(actor.run());
    }

    // Dispatcher runs on the main task — exits when all senders are dropped
    dispatcher.run().await;

    Ok(())
}
