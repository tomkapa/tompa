use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ServerToContainer {
    Execute {
        session_id: Uuid,
        system_prompt: String,
        prompt: String,
    },
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ContainerToServer {
    ExecutionResult {
        session_id: Uuid,
        output: serde_json::Value,
    },
    ExecutionFailed {
        session_id: Uuid,
        error: String,
    },
    Pong,
}
