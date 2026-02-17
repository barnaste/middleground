cli
----

A command-line interface for testing and debugging the Middleground backend. 
This tool provides streamlined authentication, WebSocket connections to conversation channels, and real-time messaging interaction.

[![Rust](https://img.shields.io/badge/rust-1.87%2B-red?logo=rust&style=for-the-badge)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL-purple?style=for-the-badge)](../LICENSE)

### Features
The CLI provides OTP-based email authentication. 
Once authenticated, developers can connect to conversation channels via WebSocket and perform full messaging operations including sending, replying, editing, and deleting messages. 
Persistent command history supports arrow key navigation and Ctrl-R reverse search for command replay. 
Terminal management ensures asynchronous WebSocket messages display cleanly above the command prompt without corrupting user input.

### Quick Start
Ensure Rust and Cargo are installed, then clone the repository.
Run the CLI from the workspace root:
```bash
cargo run -p cli -- --host http://localhost:8080 --username alice@example.com
```

For remote backends, adjust the host parameter:
```bash
cargo run -p cli --- https://api.middleground.example.com --username ...
```

The CLI initiates OTP authentication by sending an 8-digit code to your email.
Enter the code when prompted to complete authentication.
Sussion credentials are managed automatically with token refresh handling.

After authentication, the CLI presents an interactive shell with a context-aware prompt showing your username and either the backend host (when disconnected) or the first 8 characters of the conversation ID (when connected).

### Architecture
Application state is centralized in the `state` module, maintaining the backend URL, authenticated user email, authentication client with token management, terminal manager for async message display, and optional WebSocket client. 
This ensures consistent access to shared resources while maintaining proper ownership semantics.

Authentication is encapsulated in the `auth` module, providing abstraction over the backend's authentication API. 
The client handles OTP verification, manages access and refresh token lifecycles with automatic refresh, and maintains authenticated HTTP client state.

WebSocket integration uses a concurrent three-task architecture for full-duplex communication. 
The incoming handler receives and formats server messages, coordinating with the terminal manager for display. 
The outgoing handler processes user commands (send, edit, delete) and transmits them with JSON serialization. 
The display handler ensures messages appear above the prompt without corrupting user input.

Terminal management enables displaying async messages during active typing. 
The manager tracks prompt state and uses ANSI escape sequences to save the current prompt and input, clear the line for message display, and restore the prompt and partial input seamlessly.

Command parsing transforms raw input into structured command enums. 
The parser handles aliases, validates UUIDs for message and conversation identifiers, and provides clear error messages. 
Rust's enum types provide compile-time guarantees about command structure.

Configuration persistence manages user preferences in platform-appropriate locations (`~/.config/mgcli/config.toml` on Unix, `%APPDATA%\mgcli\config.toml` on Windows, `~/Library/Application Support/mgcli/config.toml` on macOS). 
Uses Serde for TOML serialization with sensible defaults and command-line argument precedence.

#### Module Organization
This crate is organized into modules:
- `main` serves as the entry point, coordinating authentication flow, managing the REPL command execution cycle, and handling graceful shutdown. 
- `auth` abstracts authentication via HTTP, managing token storage and refresh. 
- `websocket` implements the WebSocket client with the three-task concurrent architecture for message handling. 
- `terminal` provides async message display coordination using ANSI escape sequences. 
- `shellcmd` defines command enums and implements parsing with validation. 
- `state` defines global application state and manages shared resource lifecycles. 
- `config` handles configuration file management with platform-appropriate locations.

### Available Commands

**Global Commands:**
- `help` - Display comprehensive command information
- `status` - Show connection state, backend URL, authenticated user, and WebSocket status
- `clear` - Clear the terminal screen
- `exit` / `quit` - Disconnect WebSocket, logout, and shutdown

**WebSocket Commands:**
- `ws connect <conversation_id>` - Establish WebSocket connection to conversation channel by UUID
- `ws disconnect` - Close active WebSocket connection

**Messaging Commands:**
- `send <message>` (alias: `s`) - Send a new message to the current conversation
- `reply <message_id> <message>` (alias: `r`) - Reply to an existing message (creates threading)
- `edit <message_id> <new_content>` (alias: `e`) - Modify a previously sent message (own messages only)
- `delete <message_id>` (alias: `d`) - Remove a message (own messages only)

### Configuration Management

Configuration is automatically saved to platform-specific locations after first run. 
The TOML file contains backend host URL, username (email), and color preferences. 
View the config file location with `--show-config`. 
Command-line arguments override saved configuration for that session, allowing easy testing against different backends or users.

### Command History and Navigation

Command history persists across sessions in a file alongside configuration. 
Arrow keys navigate through history (up for previous, down for next). 
Ctrl-R enables reverse search: type to find matching commands, continue pressing Ctrl-R to cycle through older matches, then Enter to execute. 
This is particularly valuable for repeatedly testing message patterns or reconnecting to channels during development.

### Dependencies

**Core functionality:**
- `clap` (with derive features) - Command-line argument parsing with automatic help generation
- `tokio` (workspace) - Async runtime for concurrent operations
- `rustyline` - Readline-like functionality with history, reverse search, and completion
- `tokio-tungstenite` - WebSocket client with native TLS and tokio integration
- `reqwest` (workspace) - HTTP client for authentication API with cookie jar and JSON support
- `colored` - Terminal color output for improved readability

**Serialization and data handling:**
- `serde` / `serde_json` (workspace) - Authentication responses, WebSocket protocol, and config files
- `uuid` (workspace) - Conversation and message identifier management
- `toml` - Configuration file parsing and serialization

**Error handling and utilities:**
- `anyhow` - Rich error handling with context propagation
- `dirs` - Platform-appropriate configuration directory discovery
- `futures-util` - WebSocket stream splitting and async utilities

### License

Please see the workspace root for license information.
