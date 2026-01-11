use crate::auth::AuthClient;
use crate::websocket::WebSocketClient;

/// Application state
pub struct AppState {
    pub host: String,
    pub username: String,
    pub client: AuthClient,
    pub ws_connected: bool,
    pub ws_client: Option<WebSocketClient>,
    pub ws_channel: Option<String>,
}

impl AppState {
    pub fn new(host: String, username: String, client: AuthClient) -> Self {
        Self {
            host,
            username,
            client,
            ws_connected: false,
            ws_client: None,
            ws_channel: None,
        }
    }

    pub fn prompt(&self) -> String {
        let host_short = self
            .host
            .strip_prefix("http://")
            .or_else(|| self.host.strip_prefix("https://"))
            .unwrap_or(&self.host);

        let user_short = self
            .username
            .split('@')
            .next()
            .unwrap_or(&self.username);

        let mut parts = vec![user_short, "@", host_short];

        if self.ws_connected {
            parts.push(":ws");
            if let Some(channel) = self.ws_channel.as_ref() {
                parts.push(":");
                parts.push(channel);
            }
        }

        format!("[{}] > ", parts.join(""))
    }
}
