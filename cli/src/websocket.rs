use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

// ========================== Message Types ==========================

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OutgoingMessage {
    Send { payload: SendPayload },
    Edit { payload: EditPayload },
    Delete { payload: DeletePayload },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPayload {
    pub content: String,
    pub quoted_id: Option<Uuid>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditPayload {
    pub message_id: Uuid,
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePayload {
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
    pub write: mpsc::UnboundedSender<OutgoingMessage>,
}

impl WebSocketClient {
    pub async fn connect() {
        todo!()
    }

    pub async fn disconnect() {
        todo!()
    }

    pub async fn send() {
        todo!()
    }

    pub async fn edit() {
        todo!()
    }

    pub async fn delete() {
        todo!()
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
