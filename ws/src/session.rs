use axum::extract::ws::WebSocket;
use shared::AppState;
use uuid::Uuid;

// TODO: validate the conversation ID as one that the user is assoc with
async fn handle_socket(socket: WebSocket, user_id: Uuid, conversation_id: Uuid, state: AppState) {
    todo!()
}
