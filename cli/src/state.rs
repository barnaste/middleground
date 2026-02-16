//! Application state management.
//!
//! Centralizes all application state, including authentication, WebSocket connections, and the
//! async writer for output.

use crate::auth::AuthClient;
use crate::terminal::TerminalManager;
use crate::websocket::WebSocketClient;

/// Global application state.
///
/// Contains all mutable state that needs to be shared across the application:
/// - Authentication client with tokens
/// - Optional WebSocket client for active connections
/// - Shared terminal manager for async message output
pub struct AppState {
    /// Backend URL (e.g. "https://localhost:8080")
    pub host: String,

    /// User's email address
    pub username: String,

    /// Authentication client with token management
    pub auth_client: AuthClient,
    
    /// Terminal manager for asynchronous messaging
    pub term: TerminalManager,

    /// Active WebSocket client, if connected
    pub ws_client: Option<WebSocketClient>,
}

impl AppState {
    /// Create a new application state.
    ///
    /// # Arguments:
    /// * `host` - Backend URL
    /// * `username` - User's email address
    /// * `client` - Authenticated client with tokens
    pub fn new(host: String, username: String, client: AuthClient) -> Self {
        Self {
            host,
            username,
            auth_client: client,
            term: TerminalManager::new(),
            ws_client: None,
        }
    }

    /// Generate the command prompt string.
    ///
    /// Format: `[username@context] > `
    ///
    /// Where context is either:
    /// - The backend host (if not connected to WebSocket)
    /// - The first 8 chars of conversation ID (if connected)
    ///
    /// # Returns
    ///
    /// A formatted prompt string ready for display.
    pub fn prompt(&self) -> String {
        let host_short = self
            .host
            .strip_prefix("http://")
            .or_else(|| self.host.strip_prefix("https://"))
            .unwrap_or(&self.host);

        let user_short = self.username.split('@').next().unwrap_or(&self.username);

        // determine context (host or channel)
        let context = if let Some(client) = &self.ws_client {
            client.conversation_id().to_string()[..8].to_string()
        } else {
            host_short.to_string()
        };

        format!("[{}@{}] > ", user_short, context)
    }
}
