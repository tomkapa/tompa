use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the shared tracing subscriber.
///
/// # Formats
///
/// Controlled by `LOG_FORMAT` env var:
///
/// - **`json`** — Structured JSON, one object per line. Designed for Loki ingestion
///   via Promtail/Alloy. Fields are flattened to the top level for easy LogQL queries
///   (e.g. `{service="server"} | json | level="ERROR"`).
///
/// - **`pretty`** (or unset) — Colored, human-readable output for terminal development.
///   Includes source file and line number for quick navigation.
///
/// # Filter
///
/// `RUST_LOG` controls the filter level (default: `info`).
pub fn init_tracing(service_name: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let json = std::env::var("LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    if json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_current_span(true)
                    .with_span_list(true)
                    .flatten_event(true),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(true),
            )
            .init();
    }

    tracing::debug!(service = service_name, json, "tracing initialized");
}

/// Initialize tracing for tests.
///
/// Uses `fmt::TestWriter` so output is captured per-test and only printed
/// when a test **fails** (same behavior as `println!` under `cargo test`).
/// Pass `RUST_LOG=debug cargo test` to see more detail on failures.
///
/// Safe to call multiple times — the second call is a silent no-op
/// (the global subscriber is already set).
pub fn init_test_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_test_writer()
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false),
        )
        .try_init();
}
