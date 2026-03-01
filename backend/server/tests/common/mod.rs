#[ctor::ctor]
fn init_test_tracing() {
    shared::telemetry::init_test_tracing();
}
