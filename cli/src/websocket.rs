// TODO: reorganize this; it might be worth splitting into several modules

use anyhow::Result;
use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, protocol::CloseFrame},
};
use uuid::Uuid;

use crate::state::AppState;

// ========================== Type Aliases ==========================

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsReadHalf = futures_util::stream::SplitStream<WsStream>;
type WsWriteHalf = futures_util::stream::SplitSink<WsStream, Message>;

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

        let (sender, receiver) = ws_stream.split();

        // create communication channels
        let (message_tx, message_rx) = mpsc::unbounded_channel::<OutgoingMessage>();
        let (display_tx, display_rx) = mpsc::unbounded_channel::<DisplayCommand>();

        // spawn task to handle incoming messages (from server)
        tokio::spawn(handle_incoming(receiver, display_tx.clone()));

        // spawn task to handle outgoing messages (from user)
        tokio::spawn(handle_outgoing(sender, message_rx, display_tx.clone()));

        // spawn task to handle message displaying
        tokio::spawn(handle_display(display_rx));

        // return a handler for future messages from the user
        Ok(Self {
            conversation_id,
            tx: message_tx,
        })
    }

    pub async fn disconnect(self) {
        // signal shutdown by dropping the sender
        drop(self.tx);
    }

    pub fn send(&self, content: String, quoted_id: Option<Uuid>) -> Result<()> {
        self.tx
            .send(OutgoingMessage::Send {
                payload: SendPayload { content, quoted_id },
            })
            .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))
    }

    pub fn edit(&self, message_id: Uuid, content: String) -> Result<()> {
        self.tx
            .send(OutgoingMessage::Edit {
                payload: EditPayload {
                    message_id,
                    content,
                },
            })
            .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))
    }

    pub fn delete(&self, message_id: Uuid) -> Result<()> {
        self.tx
            .send(OutgoingMessage::Delete {
                payload: DeletePayload { message_id },
            })
            .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))
    }

    pub fn conversation_id(&self) -> Uuid {
        self.conversation_id
    }
}

// NOTE: holds the websocket sink; emits data to be displayed
// expected to handle any intermediate data issues and convert to readable fmt
async fn handle_incoming(
    mut ws_receiver: WsReadHalf,
    display_tx: mpsc::UnboundedSender<DisplayCommand>,
) {
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => match serde_json::from_str::<IncomingMessage>(&text) {
                Ok(msg) => {
                    let _ = display_tx.send(DisplayCommand::Message(msg));
                }
                Err(e) => {
                    let _ = display_tx.send(DisplayCommand::Error(format!(
                        "Failed to parse message: {}",
                        e
                    )));
                }
            },

            Ok(Message::Close(frame)) => {
                if let Some(CloseFrame { code, reason }) = frame {
                    let _ = display_tx.send(DisplayCommand::Info(format!(
                        "Connection closed: {} - {}",
                        code, reason
                    )));
                } else {
                    let _ = display_tx.send(DisplayCommand::Info("Connection closed".to_string()));
                }
                let _ = display_tx.send(DisplayCommand::Disconnected);
                break;
            }

            Ok(_) => {}

            Err(e) => {
                let _ = display_tx.send(DisplayCommand::Error(format!("WebSocket error: {}", e)));
                let _ = display_tx.send(DisplayCommand::Disconnected);
                break;
            }
        }
    }
}

// NOTE: receiving end of the buffer that WebSocketClient::write is connected to
// expected to send data through the websocket sink
async fn handle_outgoing(
    mut ws_sender: WsWriteHalf,
    mut message_rx: mpsc::UnboundedReceiver<OutgoingMessage>,
    display_tx: mpsc::UnboundedSender<DisplayCommand>,
) {
    while let Some(msg) = message_rx.recv().await {
        match serde_json::to_string(&msg) {
            Ok(json) => {
                if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                    let _ = display_tx.send(DisplayCommand::Error(format!(
                        "Failed to send message: {}",
                        e
                    )));
                    break;
                }
            }
            Err(e) => {
                let _ = display_tx.send(DisplayCommand::Error(format!(
                    "Fialed to serialize message: {}",
                    e
                )));
            }
        }
    }

    // once we reach this point, the channel is closed, so send close frame
    let _ = ws_sender.send(Message::Close(None)).await;
}

// NOTE: receives data to be displayed
// just displays it via io::stdout
//
// it feels kinda weird that we're doing prints here. maybe we should
// make ws a separate module with its own directory, and have several
// submodules with one module being display.rs?
async fn handle_display(mut display_rx: mpsc::UnboundedReceiver<DisplayCommand>) {
    while let Some(cmd) = display_rx.recv().await {
        match cmd {
            DisplayCommand::Message(msg) => {
                display_message(msg).await;
            }
            DisplayCommand::Error(err) => {
                println!("{} {}", "✗".red(), err);
            }
            DisplayCommand::Info(info) => {
                println!("{} {}", "🛈".blue(), info);
            }
            DisplayCommand::Disconnected => {
                println!("{} WebSocket connection closed", "✗".red());
                break;
            }
        }
    }
}

/// Format a UUID to show only the first 8 characters.
fn short_uuid(uuid: Uuid) -> String {
    uuid.to_string()[..8].to_string()
}

/// Format timestamp to extract the time portion.
/// Assumes the timestamp is in RFC3339 format: "YYYY-MM-DDTHH:MM:SS.SSSZ"
fn format_timestamp(timestamp: &str) -> String {
    timestamp
        .split('T')
        .nth(1)
        .and_then(|t| t.split('.').next())
        .unwrap_or(timestamp)
        .to_string()
}

/// Display an incoming message in formatted style.
async fn display_message(msg: IncomingMessage) {
    match msg {
        IncomingMessage::Send {
            message_id,
            sender_id,
            content,
            quoted_id,
            timestamp,
        } => {
            // format: timestamp [sender_id][message_id][quoted_id?] content
            let mut ids = vec![
                format!("[{}]", short_uuid(sender_id)),
                format!("[{}]", short_uuid(message_id)),
            ];

            if let Some(qid) = quoted_id {
                ids.push(format!("[{}]", short_uuid(qid)));
            }

            println!(
                "{} {} {}",
                format_timestamp(&timestamp).dimmed(),
                ids.join(""),
                content
            );
        }

        IncomingMessage::Edit {
            message_id,
            sender_id,
            content,
            timestamp,
        } => {
            // format: timestamp [sender_id][message_id] edited: content
            println!(
                "{} [{}][{}] {} {}",
                format_timestamp(&timestamp).dimmed(),
                short_uuid(sender_id),
                short_uuid(message_id),
                "edited".dimmed(),
                content
            );
        }

        IncomingMessage::Delete {
            message_id,
            sender_id,
            timestamp,
        } => {
            // format: timestamp [sender_id][message_id] == message deleted ==
            println!(
                "{} [{}][{}] {}",
                format_timestamp(&timestamp).dimmed(),
                short_uuid(sender_id),
                short_uuid(message_id),
                "== message deleted ==".red().dimmed()
            );
        }
    }
}
