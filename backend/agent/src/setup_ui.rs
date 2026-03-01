use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch};
use tracing::{error, info};

use crate::{agent_status::ConnectionStatus, dispatcher::DispatchMessage};

// Embeds backend/agent/setup-ui/dist/ into the binary at compile time.
#[derive(RustEmbed)]
#[folder = "setup-ui/dist/"]
struct Assets;

#[derive(Debug)]
pub enum SetupUiMessage {}

pub struct SetupUi {
    port: u16,
    config_path: String,
    mode: String,
    _dispatch_tx: mpsc::Sender<DispatchMessage>,
    rx: mpsc::Receiver<SetupUiMessage>,
    status_rx: watch::Receiver<ConnectionStatus>,
}

impl SetupUi {
    pub fn new(
        port: u16,
        config_path: String,
        mode: String,
        dispatch_tx: mpsc::Sender<DispatchMessage>,
        rx: mpsc::Receiver<SetupUiMessage>,
        status_rx: watch::Receiver<ConnectionStatus>,
    ) -> Self {
        Self { port, config_path, mode, _dispatch_tx: dispatch_tx, rx, status_rx }
    }

    pub async fn run(mut self) {
        info!(port = self.port, "setup_ui starting");

        let state = ApiState {
            status_rx: self.status_rx,
            config_path: self.config_path,
            mode: self.mode,
        };

        let router = Router::new()
            .route("/api/status", get(get_status))
            .route("/api/config", post(post_config))
            .fallback(serve_asset)
            .with_state(state);

        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!(port = self.port, "Failed to bind setup UI port: {e}");
                return;
            }
        };

        info!(port = self.port, "setup_ui listening");
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, router).await {
                error!("setup_ui server error: {e}");
            }
        });

        // Drain the message channel to keep the actor pattern consistent.
        while let Some(msg) = self.rx.recv().await {
            match msg {}
        }
        info!("setup_ui shutting down");
    }
}

// ── Axum state ────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ApiState {
    status_rx: watch::Receiver<ConnectionStatus>,
    config_path: String,
    mode: String,
}

// ── GET /api/status ───────────────────────────────────────────────────────────

#[derive(Serialize)]
struct StatusResponse {
    connected: bool,
    last_heartbeat: Option<u64>,
    mode: String,
}

async fn get_status(State(state): State<ApiState>) -> Json<StatusResponse> {
    let s = state.status_rx.borrow();
    Json(StatusResponse {
        connected: s.connected,
        last_heartbeat: s.last_heartbeat,
        mode: state.mode.clone(),
    })
}

// ── POST /api/config ──────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize)]
struct ConfigPayload {
    mode: String,
    server_url: String,
    api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    setup_ui_port: Option<u16>,
}

async fn post_config(
    State(state): State<ApiState>,
    Json(payload): Json<ConfigPayload>,
) -> StatusCode {
    let toml_str = match toml::to_string_pretty(&payload) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize config to TOML: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    match std::fs::write(&state.config_path, toml_str) {
        Ok(()) => {
            info!(path = %state.config_path, "Config written");
            StatusCode::OK
        }
        Err(e) => {
            error!(path = %state.config_path, "Failed to write config: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// ── Static asset serving (SPA fallback) ──────────────────────────────────────

async fn serve_asset(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(file) => {
            let mime = mime_for_path(path);
            ([(header::CONTENT_TYPE, mime)], file.data).into_response()
        }
        None => {
            // SPA fallback: unknown routes serve index.html
            match Assets::get("index.html") {
                Some(file) => {
                    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], file.data)
                        .into_response()
                }
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}

fn mime_for_path(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else {
        "application/octet-stream"
    }
}
