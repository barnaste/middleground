use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::state::AppState;

// ========================== Message Types ==========================

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum OutgoingMessage {
    Send { payload: SendPayload },
    Edit { payload: EditPayload },
    Delete { payload: DeletePayload },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SendPayload {
    pub content: String,
    pub quoted_id: Option<Uuid>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EditPayload {
    pub message_id: Uuid,
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeletePayload {
    pub message_id: Uuid,
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "snake_case"
)]
enum IncomingMessage {
    Send {
        message_id: Uuid,
        sender_id: Uuid,
        content: String,
        quoted_id: Option<Uuid>,
        timestamp: String,
    },
    Edit {
        message_id: Uuid,
        sender_id: Uuid,
        content: String,
        timestamp: String,
    },
    Delete {
        message_id: Uuid,
        sender_id: Uuid,
        timestamp: String,
    },
}

// ========================== WebSocket Client ==========================

pub struct WebSocketClient {
    conversation_id: Uuid,
    write: mpsc::UnboundedSender<OutgoingMessage>,
}

impl WebSocketClient {
    pub async fn connect(state: Arc<RwLock<AppState>>, channel: Uuid) -> Result<Self> {
        // do websocket handshake using tokio tungstenite
        todo!()
    }

    pub async fn disconnect(self, state: Arc<RwLock<AppState>>) {
        drop(self.write);
    }

    pub fn send(&self, content: String, quoted_id: Option<Uuid>) -> Result<()> {
        todo!()
    }

    pub fn edit(&self, message_id: Uuid, content: String) -> Result<()> {
        todo!()
    }

    pub fn delete(&self, message_id: Uuid) -> Result<()> {
        todo!()
    }

    pub fn conversation_id(&self) -> Uuid {
        self.conversation_id
    }
}

// NOTE: holds the websocket sink; emits data to be displayed
// expected to handle any intermediate data issues and convert to readable fmt
//
// ONCE BUFFER CLOSES: you should terminate
async fn handle_incoming() {
    todo!()
}

// NOTE: receiving end of the buffer that WebSocketClient::write is connected to
// expected to send data through the websocket sink
//
// ONCE BUFFER CLOSES: you should terminate
async fn handle_outgoing() {
    todo!()
}

// NOTE: receives data to be displayed
// just displays it via io::stdout
//
// ONCE BUFFER CLOSES: you should terminate
async fn handle_display() {
    todo!()
}
