use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, stream::SplitSink};
use redis::aio::MultiplexedConnection;
use shared::AppState;
use uuid::Uuid;

use crate::error::WsError;

// Assumes that the user and conversation are compatible.
pub async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), WsError> {
    let (mut sender, mut receiver) = socket.split();

    // set up a receiver rx that takes in all updates in the redis channel corresponding to the
    // conversation the user is connecting to
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
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
                return Err(WsError::Redis(e));
            }
        };

        let channel_name = format!("conversation:{}", conversation_id);
        if let Err(e) = con.subscribe(channel_name).await {
            return Err(WsError::Redis(e));
        }
    }

    let write_task = tokio::spawn(socket_write(sender, rx));
    let read_task = tokio::spawn(socket_read(receiver));

    read_task.await;
    write_task.abort();

    // TODO: read (via receiver) -> processing (via messages::handle) -> publish (via con)
    //   -> read (via rx in socket_write) -> write (via sender in socket_write)

    todo!()
}

// NOTE: may need the user's id
async fn socket_read(mut receiver: SplitStream<WebSocket>) {
    // read loop; converts message into something that can be processed by message module
    // we will need to publish outgoing messages via a new redis connection once it is created.
    // this should be a function within message module

    // once loop terminates, return; we should close the ws connection soon after
}

async fn socket_write(mut sender: SplitSink<WebSocket, Message>, rx: UnboundedSender<PushInfo>) {
    // create a channel, then create a thread to write back to the client
    // via the socket whenever anything is sent through the channel in rx;
    // for now we're simply converting the message from rx to a string and sending it through sender
}
