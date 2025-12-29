/// Application state
pub struct AppState {
    host: String,
    username: String,
    client: reqwest::Client,
    access_token: Option<String>,
    refresh_token: Option<String>,
    expiration: Option<u64>,
    ws_connected: bool,
    current_channel: Option<String>, // TODO: need a handle to the reading thread
}

impl AppState {
    pub fn new(host: String, username: String) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("FATAL: unable to create a HTTP client.");

        Self {
            host,
            username,
            client,
            access_token: None,
            refresh_token: None,
            expiration: None,
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

    pub fn set_auth(&mut self, access_token: String, refresh_token: String, expiration: u64) {
        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);
        self.expiration = Some(expiration);
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }
}
