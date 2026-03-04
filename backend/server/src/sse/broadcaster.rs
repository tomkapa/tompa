use dashmap::DashMap;
use serde::Serialize;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

use crate::db::new_id;

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SseEvent {
    StoryUpdated {
        story_id: Uuid,
        fields: Vec<String>,
    },
    TaskUpdated {
        task_id: Uuid,
        story_id: Uuid,
        fields: Vec<String>,
    },
    NewQuestion {
        story_id: Uuid,
        task_id: Option<Uuid>,
        round_id: Uuid,
    },
    TaskCompleted {
        task_id: Uuid,
        story_id: Uuid,
    },
    AnswersForwarded {
        story_id: Uuid,
        task_id: Option<Uuid>,
        round_id: Uuid,
    },
    RefinedDescriptionReady {
        story_id: Uuid,
        stage: String,
    },
}

impl SseEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            SseEvent::StoryUpdated { .. } => "StoryUpdated",
            SseEvent::TaskUpdated { .. } => "TaskUpdated",
            SseEvent::NewQuestion { .. } => "NewQuestion",
            SseEvent::TaskCompleted { .. } => "TaskCompleted",
            SseEvent::AnswersForwarded { .. } => "AnswersForwarded",
            SseEvent::RefinedDescriptionReady { .. } => "RefinedDescriptionReady",
        }
    }
}

pub struct SseBroadcaster {
    clients: DashMap<Uuid, Vec<(Uuid, UnboundedSender<SseEvent>)>>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    /// Subscribe to events for an org. Returns `(sender_id, receiver)`.
    /// Pass `sender_id` to `unsubscribe` when the client disconnects.
    pub fn subscribe(&self, org_id: Uuid) -> (Uuid, UnboundedReceiver<SseEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let sender_id = new_id();
        self.clients
            .entry(org_id)
            .or_default()
            .push((sender_id, tx));
        (sender_id, rx)
    }

    /// Proactively remove a specific client on disconnect.
    pub fn unsubscribe(&self, org_id: Uuid, sender_id: Uuid) {
        if let Some(mut entry) = self.clients.get_mut(&org_id) {
            entry.retain(|(id, _)| *id != sender_id);
        }
    }

    /// Send an event to all live clients for an org.
    /// Dead senders (disconnected clients) are lazily removed.
    pub fn broadcast(&self, org_id: Uuid, event: SseEvent) {
        if let Some(mut entry) = self.clients.get_mut(&org_id) {
            entry.retain(|(_, tx)| tx.send(event.clone()).is_ok());
        }
    }
}

impl Default for SseBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
