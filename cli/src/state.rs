use crate::auth::AuthClient;
use crate::websocket::WebSocketClient;

/// Application state
pub struct AppState {
    pub host: String,
    pub username: String,
    pub auth_client: AuthClient,
    pub ws_client: Option<WebSocketClient>,
}

impl AppState {
    pub fn new(host: String, username: String, client: AuthClient) -> Self {
        Self {
            host,
            username,
            auth_client: client,
            ws_client: None,
        }
    }

    pub fn prompt(&self) -> String {
        let host_short = self
            .host
            .strip_prefix("http://")
            .or_else(|| self.host.strip_prefix("https://"))
            .unwrap_or(&self.host);

        let user_short = self.username.split('@').next().unwrap_or(&self.username);

        format!(
            "[{}@{}{}] > ",
            user_short,
            host_short,
            if let Some(client) = &self.ws_client {
                format!(":{}", &client.conversation_id().to_string()[..8])
            } else {
                String::new()
            }
        )
    }
}
