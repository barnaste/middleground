use anyhow::Result;
use futures_util::{
    StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
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

// ========================== Display Commands ==========================

enum DisplayCommand {
    Message(IncomingMessage),
    Error(String),
    Info(String),
    Disconnected,
}

// ========================== WebSocket Client ==========================

pub struct WebSocketClient {
    conversation_id: Uuid,
    tx: mpsc::UnboundedSender<OutgoingMessage>,
}

impl WebSocketClient {
    pub async fn connect(state: Arc<RwLock<AppState>>, conversation_id: Uuid) -> Result<Self> {
        // get authorization headers
        let mut state_write = state.write().await;
        let headers = state_write.auth_client.get_authorization_header().await;
        drop(state_write);

        // determine connection URL
        let state_read = state.read().await;
        let base_url = state_read
            .host
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = format!("{}/ws?conversation_id={}", base_url, conversation_id);
        drop(state_read);

        let mut request = http::Request::builder()
            .uri(url)
            .body(())
            .map_err(|e| anyhow::anyhow!("Failed to build request: {}", e))?;
        request.headers_mut().extend(headers);

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket connection failed: {}", e))?;

        let (write, read) = ws_stream.split();

        // create communication channels
        let (message_tx, message_rx) = mpsc::unbounded_channel::<OutgoingMessage>();
        let (display_tx, display_rx) = mpsc::unbounded_channel::<DisplayCommand>();

        // spawn task to handle incoming messages (from server)
        tokio::spawn(handle_incoming(read, display_tx.clone()));
        
        // spawn task to handle outgoing messages (from user)
        tokio::spawn(handle_outgoing(write, message_rx, display_tx.clone()));

        // spawn task to handle message displaying
        tokio::spawn(handle_display(display_rx, state.clone()));

        // return a handler for future messages from the user
        Ok(Self{
            conversation_id,
            tx: message_tx,
        })
    }

    pub async fn disconnect(self, state: Arc<RwLock<AppState>>) {
        // signal shutdown by dropping the sender
        drop(self.tx);
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
async fn handle_incoming<T>(
    mut read: SplitStream<T>,
    display_tx: mpsc::UnboundedSender<DisplayCommand>,
) where
    T: Sized + futures_util::Sink<Message>,
{
    todo!()
}

// NOTE: receiving end of the buffer that WebSocketClient::write is connected to
// expected to send data through the websocket sink
//
// ONCE BUFFER CLOSES: you should terminate
async fn handle_outgoing<T>(
    mut write: SplitSink<T, Message>,
    mut message_rx: mpsc::UnboundedReceiver<OutgoingMessage>,
    display_tx: mpsc::UnboundedSender<DisplayCommand>,
) where
    T: Sized + futures_util::Sink<Message>,
{
    todo!()
}

// NOTE: receives data to be displayed
// just displays it via io::stdout
//
// ONCE BUFFER CLOSES: you should terminate
async fn handle_display(
    mut display_rx: mpsc::UnboundedReceiver<DisplayCommand>,
    state: Arc<RwLock<AppState>>,
) {
    todo!()
}
