use axum::extract::ws::{Message, WebSocket};
use futures::{
    StreamExt,
    stream::{SplitSink, SplitStream},
};
use shared::AppState;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    error::WsError,
    messages::{IncomingMessage, publish_msg},
};

// Assumes that the user and conversation are compatible.
pub async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), WsError> {
    let (sender, mut receiver) = socket.split();

    // set up a receiver rx that takes in all updates in the redis channel corresponding to the
    // conversation the user is connecting to
    let (tx, rx) = mpsc::unbounded_channel();
    {
        // TODO: we assume RESP3 is set for redis
        let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);

        let mut con = match state
            .redis
            .clone()
            .get_multiplexed_async_connection_with_config(&config)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create Redis connection: {}", e);
                return Err(WsError::Redis(e));
            }
        };

        let channel_name = format!("conversation:{}", conversation_id);
        if let Err(e) = con.subscribe(&channel_name).await {
            tracing::error!("Failed to subscribe to {}: {}", channel_name, e);
            return Err(WsError::Redis(e));
        }
    }

    let write_task = tokio::spawn(socket_write(sender, rx));
    let read_task = tokio::spawn(socket_read(
        receiver,
        state.clone(),
        user_id,
        conversation_id,
    ));

    if let Err(e) = read_task.await {
        tracing::error!("Failed to join after WebSocket connection closed: {}", e);
    };

    write_task.abort();

    // TODO: read (via receiver) -> processing (via messages::handle) -> publish (via publish_message)
    //   -> read (via rx in socket_write) -> write (via sender in socket_write)

    todo!()
}

// TODO: maybe we should wrap this into a websocket state (AppState -> WsState)?
async fn socket_read(
    mut receiver: SplitStream<WebSocket>,
    state: AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) {
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        };

        match msg {
            // NOTE: we could return a message to the user indicating that their
            // message failed, but we would need the sender if we wanted to avoid
            // broadcasting the message to every connected user...
            Message::Text(text) => {
                // parse incoming message
                let incoming: IncomingMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        tracing::error!("Failed to parse message: {}", e);
                        continue;
                    }
                };

                let outgoing = match incoming
                    .handle(user_id, conversation_id, &state.db_pool)
                    .await
                {
                    Ok(out) => out,
                    Err(e) => {
                        tracing::error!("Failed to handle message: {}", e);
                        continue;
                    }
                };

                if let Err(e) = publish_msg(conversation_id, outgoing, &state.redis).await {
                    tracing::error!("Failed to broadcast message: {}", e);
                }
            }

            Message::Close(close) => {
                if let Some(frame) = close {
                    tracing::info!("Client closed connection: {} {}", frame.code, frame.reason);
                } else {
                    tracing::info!("Client closed connection");
                }
                break;
            }

            _ => {
                // ignore other message types (Binary, Ping); the former is not supported, and the
                // latter is automatically responded to by the server
            }
        }
    }

    // once loop terminates, return; we should close the ws connection soon after
}

async fn socket_write(
    sender: SplitSink<WebSocket, Message>,
    rx: mpsc::UnboundedReceiver<redis::PushInfo>,
) {
    // create a channel, then create a thread to write back to the client
    // via the socket whenever anything is sent through the channel in rx;
    // for now we're simply converting the message from rx to a string and sending it through sender
}
