use axum::extract::ws::{Message, WebSocket};
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use shared::AppState;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    error::WsResult,
    messages::{IncomingMessage, publish_msg},
};

// Assumes that the user and conversation are compatible.
pub async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> WsResult<()> {
    let (sender, receiver) = socket.split();

    // set up a receiver rx that takes in all updates in the redis channel corresponding to the
    // conversation the user is connecting to
    let (tx, rx) = mpsc::unbounded_channel();
    {
        // TODO: we assume RESP3 is set for redis
        let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);

        let mut conn = state
            .redis
            .clone()
            .get_multiplexed_async_connection_with_config(&config)
            .await
            .inspect_err(|e| tracing::error!(error = %e, "Failed to create Redis connection"))?;

        let channel_name = format!("conversation:{}", conversation_id);
        conn.subscribe(&channel_name).await.inspect_err(
            |e| tracing::error!(error = %e, "Failed to subscribe to {}", channel_name),
        )?;
    }

    let write_task = tokio::spawn(socket_write(sender, rx));
    let read_task = tokio::spawn(socket_read(
        receiver,
        state.clone(),
        user_id,
        conversation_id,
    ));

    let _ = read_task.await.inspect_err(
        |e| tracing::error!(error = %e, "Failed to join after WebSocket connection closed"),
    );
    write_task.abort();

    Ok(())
}

async fn socket_read(
    mut receiver: SplitStream<WebSocket>,
    state: AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> WsResult<()> {
    while let Some(msg) = receiver.next().await {
        let msg = msg.inspect_err(|e| tracing::error!(error = %e, "WebSocket error"))?;

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

                publish_msg(conversation_id, outgoing, &state.redis)
                    .await
                    .inspect_err(|e| tracing::error!(error = %e, "Failed to broadcast message"))
                    .ok();
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

    Ok(())
}

/// Reads from a receiver subscribed to a Redis channel, and directs the data into the provided
/// sender, which is expected to be the WebSocket sink.
async fn socket_write(
    mut sender: SplitSink<WebSocket, Message>,
    mut rx: mpsc::UnboundedReceiver<redis::PushInfo>,
) {
    // create a channel, then create a thread to write back to the client
    // via the socket whenever anything is sent through the channel in rx;
    // for now we're simply converting the message from rx to a string and sending it through sender
    while let Some(push_info) = rx.recv().await {
        // we only handle push information that encodes a message;
        // that data should already be serialized, to we just forward it
        if let Some(msg) = redis::Msg::from_push_info(push_info) {
            let payload: redis::RedisResult<String> = msg.get_payload();

            if let Ok(msg) = payload {
                sender
                    .send(Message::Text(msg.into()))
                    .await
                    .inspect_err(|e| tracing::error!(error = %e, "Failed to send message"));
            } else {
                tracing::error!(
                error = %payload.unwrap_err(),
                    "Failed reading from Redis channel"
                );
            }
        }
    }
}
