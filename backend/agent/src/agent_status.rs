use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Default)]
pub struct ConnectionStatus {
    pub connected: bool,
    /// Unix epoch seconds of the last server ping received.
    pub last_heartbeat: Option<u64>,
}

impl ConnectionStatus {
    pub fn connected() -> Self {
        Self { connected: true, last_heartbeat: None }
    }

    pub fn disconnected() -> Self {
        Self { connected: false, last_heartbeat: None }
    }

    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());
    }
}
