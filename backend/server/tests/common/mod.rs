use std::sync::Mutex;

use testcontainers::ImageExt;
use testcontainers::runners::SyncRunner;
use testcontainers_modules::postgres::Postgres;

/// Container handle kept alive for the test binary's lifetime.
/// Dropped by `#[dtor]` to ensure the container is removed after tests.
static PG_CONTAINER: Mutex<Option<testcontainers::Container<Postgres>>> = Mutex::new(None);

#[ctor::ctor]
fn init_test_env() {
    shared::telemetry::init_test_tracing();

    // CI sets DATABASE_URL — skip container startup
    if std::env::var("DATABASE_URL").is_ok() {
        return;
    }

    let container = Postgres::default()
        .with_db_name("test")
        .with_user("test")
        .with_password("test")
        .with_tag("16-alpine")
        .start()
        .expect("failed to start Postgres testcontainer — is Docker running?");

    let host = container.get_host().expect("failed to get container host");
    let port = container
        .get_host_port_ipv4(5432)
        .expect("failed to get container port");
    let url = format!("postgres://test:test@{host}:{port}/test");

    // SAFETY: #[ctor] runs before main(), single-threaded — no data race.
    unsafe { std::env::set_var("DATABASE_URL", &url) };

    *PG_CONTAINER.lock().expect("PG_CONTAINER poisoned") = Some(container);
}

#[ctor::dtor]
fn cleanup_test_env() {
    if let Ok(mut guard) = PG_CONTAINER.lock()
        && let Some(container) = guard.take()
    {
        let id = container.id().to_string();
        // Prevent Container's Drop from running — it panics in dtor
        // because the async runtime is already torn down.
        std::mem::forget(container);
        // Use docker CLI directly for reliable cleanup.
        let _ = std::process::Command::new("docker")
            .args(["rm", "-f", &id])
            .output();
    }
}
