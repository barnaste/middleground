//! # Middleground CLI
//!
//! A command-line interface for testing and debugging the Middleground backend.
//! Provides authentication, WebSocket connections, and real-time messaging capabilities.
//!
//! ## Features
//!
//! - OTP-based authentication
//! - WebSocket conversation connections
//! - Configuration file saving preferences (e.g. username, host)
//! - Auto-completion via TAB with inline hints
//! - Persistent command history with arrow key navigation and reverse search
//! - Async message display with terminal management
//!
//! Features will continuously be added as the backend expands.
//!
//! ## Usage
//!
//! ```bash
//! cargo run -- --host https://localhost:8080 --username user@example.com
//! ```

// TODO: add auto-completion, inline hints, configuration file, persistent command history, ctl-r

mod auth;
mod shellcmd;
mod state;
mod terminal;
mod websocket;

use anyhow::Result;
use auth::AuthClient;
use clap::Parser;
use colored::Colorize;
use shellcmd::ShellCommand;
use state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(
    name = "middleground-cli",
    version,
    about = "CLI tool for testing and debugging the Middleground backend",
    long_about = None
)]
struct Cli {
    /// Backend host URL
    #[arg(short = 'H', long, default_value = "http://localhost:8080")]
    host: String,

    /// Contact information for authentication
    #[arg(short, long)]
    username: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Show config file path and exit
    #[arg(long)]
    show_config: bool,
}

/// Display help information for all available commands
fn show_help() {
    println!("{}", "┌────────────────────────────────────────┐".cyan());
    println!("{}", "│          AVAILABLE COMMANDS            │".cyan());
    println!("{}", "└────────────────────────────────────────┘".cyan());
    println!();

    println!("  {}", "Global Commands".bright_yellow().bold());
    println!("    help  {}", "Show this help message".dimmed());
    println!(
        "    status  {}",
        "Show connection and session status".dimmed()
    );
    println!("    clear  {}", "Clear the terminal screen".dimmed());
    println!("    exit/quit  {}", "Exit the CLI".dimmed());
    println!();

    println!("  {}", "WebSocket Commands".bright_yellow().bold());
    println!(
        "    ws connect <channel>  {}",
        "Connect to a conversation channel".dimmed()
    );
    println!(
        "    ws disconnect  {}",
        "Disconnect from current channel".dimmed()
    );
    println!();

    println!("  {}", "Messaging Commands".bright_yellow().bold());
    println!("    send <msg>  {}", "Send a message (alias: s)".dimmed());
    println!(
        "    reply <msg_id> <msg>  {}",
        "Reply to a message (alias: r)".dimmed()
    );
    println!(
        "    edit <msg_id> <msg>  {}",
        "Edit your message (alias: e)".dimmed()
    );
    println!(
        "    delete <msg_id>  {}",
        "Delete your message (alias: d)".dimmed()
    );
    println!();

    println!("  {}", "Tips".bright_cyan().italic());
    println!(
        "    • Press {} for command completion",
        "TAB".bright_yellow()
    );
    println!(
        "    • Press {} to search command history",
        "Ctrl-R".bright_yellow()
    );
    println!(
        "    • Use {} to navigate history",
        "↑/↓ arrows".bright_yellow()
    );
    println!("    • Commands show {} as you type", "hints".dimmed());
    println!(
        "    • Message IDs are shown in square brackets: {}",
        "[msg:12345678]".dimmed()
    );
    println!(
        "    • Type {} anytime to see this help",
        "help".bright_yellow()
    );
    println!();
}

/// Display current connection status and session information
async fn show_status(state: &Arc<RwLock<AppState>>) {
    let state = state.read().await;

    println!();
    println!("{}", "┌────────────────────────────────────────┐".cyan());
    println!("{}", "│           CONNECTION STATUS            │".cyan());
    println!("{}", "└────────────────────────────────────────┘".cyan());
    println!();

    // note that we are always connected to the backend, as we require
    // authentication prior to entering the main REPL
    println!("  {}", "Backend".bright_yellow().bold());
    println!("    Status:   {} Connected", "✓".green());
    println!("    URL:      {}", state.host.bright_blue());
    println!("    User:     {}", state.username.bright_blue());

    println!("  {}", "WebSocket".bright_yellow().bold());
    if let Some(client) = &state.ws_client {
        println!("    Status:   {} Connected", "✓".green());
        println!(
            "    Channel:  {}",
            client.conversation_id().to_string().bright_blue()
        );
    } else {
        println!("    Status:   {} Disconnected", "✗".red());
        println!(
            "    Channel:  {}",
            "Use 'ws connect <channel>' to connect".dimmed()
        );
    }
    println!();
}

/// Handle a parsed shell command
///
/// Executes the appropriate action for each command type. Most commands are non-blocking and
/// return quickly, with the exception of WebSocket operations.
async fn handle_command(command: ShellCommand, state: Arc<RwLock<AppState>>) -> Result<()> {
    // NOTE: handle_command is only ever called _after_ we have received
    // user input, and only prompts another instruction after it completes.
    // As a consequence, it does not need to rely on the state's terminal
    // manager to print messages.

    match command {
        ShellCommand::Help => {
            show_help();
        }

        ShellCommand::Status => {
            show_status(&state).await;
        }

        ShellCommand::Clear => {
            println!(
                "{}{}",
                terminal::ansi::CLEAR_SCREEN,
                terminal::ansi::CURSOR_HOME
            );
        }

        ShellCommand::Exit => {
            let mut state_write = state.write().await;

            // disconnect the WebSocket client if it's currently connected
            if let Some(client) = state_write.ws_client.take() {
                client.disconnect().await;
            }

            state_write.auth_client.logout().await?;

            println!("{} Logged out successfully. Goodbye!", "✓".green());
            std::process::exit(0);
        }

        ShellCommand::WsConnect(channel) => {
            // avoid connecting if you're already connected
            let state_read = state.read().await;
            if state_read.ws_client.is_some() {
                println!(
                    "{} Already connected. Use 'ws disconnect' first.",
                    "✗".red()
                );
                return Ok(());
            }
            let term = state_read.term.clone();
            drop(state_read);

            println!(
                "{} Connecting to channel {}...",
                "⟳".cyan(),
                channel.to_string()[..8].bright_blue()
            );

            // the ws client should be concerned with handling ws connections, not with how the
            // cli tracks its state, which is why state update is external w.r.t. connect()
            match websocket::WebSocketClient::connect(state.clone(), channel, term).await {
                Ok(client) => {
                    let mut state_write = state.write().await;
                    state_write.ws_client = Some(client);
                    println!("{} Connected successfully!", "✓".green());
                }
                Err(e) => println!("{} Failed to connect, {}", "✗".red(), e),
            }
        }

        ShellCommand::WsDisconnect => {
            let client = {
                let mut state_write = state.write().await;
                state_write.ws_client.take()
            };

            if let Some(client) = client {
                client.disconnect().await;
                println!("{} Disconnected from channel", "✓".green());
            } else {
                println!(
                    "{} Not connected. Use 'ws connect <channel> first",
                    "✗".red()
                );
            }
        }

        ShellCommand::Send(content) => {
            let state_read = state.read().await;
            if let Some(client) = &state_read.ws_client {
                if let Err(e) = client.send(content, None) {
                    println!("{} Failed to send message: {}", "✗".red(), e);
                }
            } else {
                println!(
                    "{} Not connected. Use 'ws connect <channel> first",
                    "✗".red()
                );
            }
        }

        ShellCommand::Reply(id, content) => {
            let state_read = state.read().await;
            if let Some(client) = &state_read.ws_client {
                if let Err(e) = client.send(content, Some(id)) {
                    println!("{} Failed to send reply: {}", "✗".red(), e);
                }
            } else {
                println!(
                    "{} Not connected. Use 'ws connect <channel> first",
                    "✗".red()
                );
            }
        }

        ShellCommand::Edit(id, content) => {
            let state_read = state.read().await;
            if let Some(client) = &state_read.ws_client {
                if let Err(e) = client.edit(id, content) {
                    println!("{} Failed to edit message: {}", "✗".red(), e);
                }
            } else {
                println!(
                    "{} Not connected. Use 'ws connect <channel> first",
                    "✗".red()
                );
            }
        }

        ShellCommand::Delete(id) => {
            let state_read = state.read().await;
            if let Some(client) = &state_read.ws_client {
                if let Err(e) = client.delete(id) {
                    println!("{} Failed to delete message: {}", "✗".red(), e);
                }
            } else {
                println!(
                    "{} Not connected. Use 'ws connect <channel> first",
                    "✗".red()
                );
            }
        }

        ShellCommand::Unknown(err) => {
            if !err.is_empty() {
                println!("{} {}", "✗".red(), err);
                println!("Type {} for available commands", "'help'".bright_yellow());
            }
        }
    }

    Ok(())
}

/// Main REPL (Read-Eval-Print Loop)
///
/// Displays a prompt, reads user input, parses commands, and executes them.
/// Continues until the user exits or an unrecoverable error occurs.
///
/// Features:
/// - Persistent command history across sessionss
/// - Searchable history with Ctrl-R
/// - Auto-completion with TAB
/// - Inline hints as you type
async fn run(state: Arc<RwLock<AppState>>) -> Result<()> {
    // TODO: can have hints and auto-completion if desired
    use rustyline::DefaultEditor;
    use rustyline::error::ReadlineError;

    println!();
    println!(
        "{}",
        "╔════════════════════════════════════════╗".bright_cyan()
    );
    println!(
        "{}",
        "║   Middleground CLI v1.0.0              ║".bright_cyan()
    );
    println!(
        "{}",
        "║   Backend Testing & Debugging Tool     ║".bright_cyan()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════╝".bright_cyan()
    );
    println!();

    // print entry data
    let state_read = state.read().await;
    println!(
        "  {}  {}",
        "Backend".bright_yellow().bold(),
        state_read.host.bright_blue()
    );
    println!(
        "  {}     {}",
        "User".bright_yellow().bold(),
        state_read.username.bright_blue()
    );
    println!();
    println!("  Type {} for available commands", "'help'".bright_yellow());
    println!("  Press {} for auto-completion", "TAB".bright_yellow());
    println!(
        "  Press {} to search command history",
        "Ctrl-R".bright_yellow()
    );
    println!();
    drop(state_read);

    // TODO: set up readline with auto-completion and history
    let mut rl = DefaultEditor::new()?;

    loop {
        // get current prompt
        let prompt = {
            // update the terminal manager with the current prompt
            let state_read = state.read().await;
            let prompt = state_read.prompt();
            state_read
                .term
                .lock()
                .await
                .set_prompt(prompt.clone())
                .await;
            prompt
        };

        // read user input
        println!();
        let command = match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                ShellCommand::parse(&line)
            }
            Err(ReadlineError::Interrupted) => {
                // TODO: merge these and save history when you exit
                println!("CTRL-C");
                ShellCommand::Exit
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                ShellCommand::Exit
            }
            Err(e) => {
                println!("{} {}", "✗".red(), e);
                let state_read = state.read().await;
                state_read.term.lock().await.unset_prompt().await;
                continue;
            }
        };

        // we have finished reading the user's prompt, so we should
        // unset the terminal state; it's okay if we receive asynchronous
        // messages now. notice that on error we skip this unset, which
        // necessitates the call to unset in the corresponding branch above
        {
            let state_read = state.read().await;
            state_read.term.lock().await.unset_prompt().await;
        }

        // execute the corresponding command
        if let Err(e) = handle_command(command, state.clone()).await {
            println!("{} {}", "✗".red(), e)
        }
    }
}

/// Perform OTP-based authentication flow
///
/// Sends an OTP to the user's email and prompts for the verification code.
/// Retries automatically on failure until successful authentication.
async fn perform_otp_login(host: &str, contact: &str) -> AuthClient {
    let mut auth_client = AuthClient::new(host);

    println!();
    println!("{} Sending OTP to {}", "⟳".cyan(), contact.bright_blue());

    // send OTP with retry logic
    while auth_client.send_otp(contact).await.is_err() {
        println!("{} Unable to send OTP for verification.", "✗".red());
        println!("  Retrying in 60 seconds...");
        std::thread::sleep(std::time::Duration::from_secs(60));
    }

    println!("{} OTP Sent! Check your email.", "✓".green());
    println!();

    loop {
        use std::io::{self, Write};

        print!("Enter 8-digit code: ");
        io::stdout().flush().unwrap();

        let mut otp = String::new();
        io::stdin().read_line(&mut otp).unwrap();

        println!("{} Verifying...", "⟳".cyan());

        if auth_client.verify_otp(contact, otp.trim()).await.is_ok() {
            println!("{} Verification successful!", "✓".green());
            println!();
            return auth_client;
        } else {
            println!("{} Verification failed, try again.", "✗".red());
            println!();
        };
    }
}

#[tokio::main]
async fn main() {
    let mut args = Cli::parse();

    // TODO: handle show-config flag and load config

    // disable colors if requested
    if args.no_color {
        colored::control::set_override(false);
    }

    // acquire the user's contact if not provided
    let username = if let Some(u) = args.username {
        u
    } else {
        use std::io::{self, Write};

        println!();
        print!("Email: ");
        io::stdout().flush().unwrap();

        let mut email = String::new();
        io::stdin()
            .read_line(&mut email)
            .expect("Failed to read email");

        email
    };

    // handle log-in using OTP
    let client = perform_otp_login(&args.host, &username).await;

    // create application state and run main REPL
    let state = AppState::new(args.host, username, client);
    if let Err(e) = run(Arc::new(RwLock::new(state))).await {
        eprintln!("{} Fatal error: {}", "✗".red(), e);
        std::process::exit(1);
    }
}
