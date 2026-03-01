use anyhow::Result;
use dashmap::DashMap;
use shared::messages::ServerToContainer;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Sender half of the per-connection outbound channel.
///
/// The write task holds the receiver and forwards serialised
/// `ServerToContainer` messages onto the WebSocket stream.
pub type WebSocketSender = mpsc::UnboundedSender<ServerToContainer>;

/// Interface for managing active container WebSocket connections.
pub trait ConnectionRegistry: Send + Sync {
    /// Register a new connection identified by its API-key UUID.
    fn register(&self, key_id: Uuid, sender: WebSocketSender);
    /// Remove a connection (called on disconnect / reconnect cleanup).
    fn unregister(&self, key_id: Uuid);
    /// Send a message to the container identified by `key_id`.
    fn send_to(&self, key_id: Uuid, msg: ServerToContainer) -> Result<()>;
    /// Return `true` if a live connection exists for `key_id`.
    fn is_connected(&self, key_id: Uuid) -> bool;
}

/// Production implementation backed by a lock-free concurrent hash map.
pub struct DashMapRegistry {
    connections: DashMap<Uuid, WebSocketSender>,
}

impl DashMapRegistry {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }
}

impl Default for DashMapRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionRegistry for DashMapRegistry {
    fn register(&self, key_id: Uuid, sender: WebSocketSender) {
        self.connections.insert(key_id, sender);
    }

    fn unregister(&self, key_id: Uuid) {
        self.connections.remove(&key_id);
    }

    fn send_to(&self, key_id: Uuid, msg: ServerToContainer) -> Result<()> {
        match self.connections.get(&key_id) {
            Some(sender) => sender.send(msg).map_err(|e| anyhow::anyhow!("{e}")),
            None => Err(anyhow::anyhow!("no connection for key {key_id}")),
        }
    }

    fn is_connected(&self, key_id: Uuid) -> bool {
        match self.connections.get(&key_id) {
            Some(s) => !s.is_closed(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::messages::ServerToContainer;

    fn make_registry() -> DashMapRegistry {
        DashMapRegistry::new()
    }

    #[test]
    fn register_and_is_connected() {
        let reg = make_registry();
        let id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel::<ServerToContainer>();
        assert!(!reg.is_connected(id));
        reg.register(id, tx);
        assert!(reg.is_connected(id));
    }

    #[test]
    fn unregister_removes_connection() {
        let reg = make_registry();
        let id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel::<ServerToContainer>();
        reg.register(id, tx);
        reg.unregister(id);
        assert!(!reg.is_connected(id));
    }

    #[test]
    fn send_to_delivers_message() {
        let reg = make_registry();
        let id = Uuid::now_v7();
        let (tx, mut rx) = mpsc::unbounded_channel::<ServerToContainer>();
        reg.register(id, tx);
        reg.send_to(id, ServerToContainer::Ping).unwrap();
        assert!(matches!(rx.try_recv().unwrap(), ServerToContainer::Ping));
    }

    #[test]
    fn send_to_unknown_key_returns_error() {
        let reg = make_registry();
        let id = Uuid::now_v7();
        assert!(reg.send_to(id, ServerToContainer::Ping).is_err());
    }

    #[test]
    fn send_to_closed_channel_returns_error() {
        let reg = make_registry();
        let id = Uuid::now_v7();
        let (tx, rx) = mpsc::unbounded_channel::<ServerToContainer>();
        reg.register(id, tx);
        drop(rx); // close the receiver
        assert!(reg.send_to(id, ServerToContainer::Ping).is_err());
    }

    #[test]
    fn is_connected_false_after_receiver_dropped() {
        let reg = make_registry();
        let id = Uuid::now_v7();
        let (tx, rx) = mpsc::unbounded_channel::<ServerToContainer>();
        reg.register(id, tx);
        drop(rx);
        assert!(!reg.is_connected(id));
    }

    #[test]
    fn multiple_connections_are_independent() {
        let reg = make_registry();
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        let (tx1, _rx1) = mpsc::unbounded_channel::<ServerToContainer>();
        let (tx2, _rx2) = mpsc::unbounded_channel::<ServerToContainer>();
        reg.register(id1, tx1);
        reg.register(id2, tx2);
        assert!(reg.is_connected(id1));
        assert!(reg.is_connected(id2));
        reg.unregister(id1);
        assert!(!reg.is_connected(id1));
        assert!(reg.is_connected(id2));
    }

    #[test]
    fn register_replaces_existing_connection() {
        let reg = make_registry();
        let id = Uuid::now_v7();
        let (tx1, rx1) = mpsc::unbounded_channel::<ServerToContainer>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<ServerToContainer>();
        reg.register(id, tx1);
        reg.register(id, tx2); // replaces tx1
        drop(rx1); // old receiver gone, but shouldn't affect is_connected
        reg.send_to(id, ServerToContainer::Ping).unwrap();
        assert!(matches!(rx2.try_recv().unwrap(), ServerToContainer::Ping));
    }
}
