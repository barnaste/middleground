use crate::auth::AuthClient;

/// Application state
pub struct AppState {
    host: String,
    username: String,
    client: AuthClient,
    ws_connected: bool,
    current_channel: Option<String>, // TODO: need a handle to the reading thread
}

impl AppState {
    pub fn new(host: String, username: String, client: AuthClient) -> Self {
        Self {
            host,
            username,
            client,
            ws_connected: false,
            current_channel: None,
        }
    }

    pub fn prompt(&self) -> String {
        let host_short = self
            .host
            .strip_prefix("http://")
            .or_else(|| self.host.strip_prefix("https://"))
            .unwrap_or(&self.host);

        let mut parts = vec![&self.username, "@", host_short];

        if self.ws_connected {
            parts.push(":ws");
            if let Some(channel) = self.current_channel.as_ref() {
                parts.push(":");
                parts.push(channel);
            }
        }

        format!("[{}] >", parts.join(""))
    }

    pub fn get_client(&self) -> &AuthClient {
        &self.client
    }
}
