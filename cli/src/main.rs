// TODO: see the README.md file for additional specifications
//  => websocket features (to be done AFTER the server websocket implementation is complete)
mod auth;
mod shellcmd;
mod state;

use anyhow::Result;
use auth::AuthClient;
use clap::Parser;
use colored::Colorize;
use shellcmd::ShellCommand;
use state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Backend testing and debugging CLI
#[derive(Parser, Debug)]
#[command(version, about)]
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
}

fn show_help() {
    println!("{}", "Available Commands:".bright_cyan().bold());
    println!();

    println!("{}", "Global:".bright_yellow());
    println!("  help                    Show help for all commands");
    println!("  status                  Show connection and session status");
    println!("  clear                   Clear screen");
    println!("  exit/quit               Exit the CLI");
    println!();

    println!("{}", "WebSocket:".bright_yellow());
    println!("  ws connect [url]        Connect to WebSocket endpoint");
    println!("  ws disconnect           Disconnect from WebSocket");
    println!("  ws status               Show WebSocket connection status");
    println!();

    println!("{}", "Messaging:".bright_yellow());
    println!("  subscribe <channel>     Subscribe to a channel (starts live feed)");
    println!("  unsubscribe             Unsubscribe from channel");
    println!("  send <msg>              Send a message to current channel");
    println!("  reply <msgId> <msg>     Reply to a message in current channel");
    println!("  edit <msgId> <msg>      Edit an owned message");
    println!("  delete <msgId>          Delete an owned message");
    println!("  messages [limit]        Show recent messages");
    println!();
}

async fn show_status(state: &Arc<RwLock<AppState>>) {
    let state = state.read().await;

    // note that we are always connected to the backend, as we require authentication prior to
    // entering the main REPL
    println!("{}", "Connection Status:".bright_cyan().bold());
    println!("  Backend:    {} {}", "✓".green(), state.host);

    println!(
        "  WebSocket:  {} {}",
        if state.ws_connected {
            "✓".green()
        } else {
            "✗".red()
        },
        state.host.replace("http", "ws") + "/ws",
    );

    // TODO: add more details once websockets are implemented
}

async fn handle_command(command: ShellCommand, state: Arc<RwLock<AppState>>) -> Result<()> {
    match command {
        ShellCommand::Help => {
            show_help();
        }

        ShellCommand::Status => {
            show_status(&state).await;
        }

        ShellCommand::Clear => {
            println!("\x1B[2J\x1B[1;1H")
        }

        ShellCommand::Exit => {
            let mut state_write = state.write().await;
            state_write.client.logout().await?;

            println!("{} Goodbye!", "✓".green());
            std::process::exit(0);
        }

        ShellCommand::Unknown(cmd) => {
            if !cmd.is_empty() {
                println!("{} Unknown command: '{}'", "✗".red(), cmd);
                println!("Type {} for available commands\n", "'help'".bright_yellow());
            }
        }

        _ => {
            println!("Feature is not yet implemented.");
        }
    }

    Ok(())
}

async fn run(state: Arc<RwLock<AppState>>) -> Result<()> {
    // TODO: can have hints and auto-completion if desired
    use rustyline::DefaultEditor;
    use rustyline::error::ReadlineError;

    println!(
        "{}",
        "┌─────────────────────────────────────────┐\n\
         │  Middleground CLI v1.0.0                │\n\
         │  Backend Testing & Debugging Tool       │\n\
         └─────────────────────────────────────────┘"
            .bright_cyan()
    );

    // print entry data
    let state_read = state.read().await;
    println!("Connected to: {}", state_read.host.bright_blue());
    println!("User: {}", state_read.username.bright_blue());
    println!("Type {} for available commands\n", "'help'".bright_yellow());
    drop(state_read);

    let mut rl = DefaultEditor::new()?;

    loop {
        let prompt = {
            let state_read = state.read().await;
            state_read.prompt()
        };

        // read the user's prompt
        let mut command;
        match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                command = ShellCommand::parse(&line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                command = ShellCommand::Exit;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                command = ShellCommand::Exit;
            }
            Err(e) => {
                println!("{} {}", "✗".red(), e);
                continue;
            }
        }

        // execute the corresponding command
        if let Err(e) = handle_command(command, state.clone()).await {
            println!("{} {}", "✗".red(), e)
        }
    }
}

async fn perform_otp_login(host: &str, contact: &str) -> AuthClient {
    let mut auth_client = AuthClient::new(host);
    println!("Sending OTP to {}", contact.blue());

    while auth_client.send_otp(contact).await.is_err() {
        println!("{} Unable to send OTP for verification.", "✗".red());
        println!("Retrying in 60 seconds...");
        std::thread::sleep(std::time::Duration::new(60, 0));
    }

    println!("{} OTP Sent! Check your email.", "✓".green());
    println!("\nEnter the 8 digit code: ");

    loop {
        let mut otp = String::new();
        std::io::stdin().read_line(&mut otp).unwrap();
        println!("Verifying OTP...");

        if auth_client.verify_otp(contact, otp.trim()).await.is_ok() {
            println!("{} Verification successful!\n", "✓".green());
            return auth_client;
        } else {
            println!("{} Verification failed, try again.", "✗".red());
        };
    }
}

#[tokio::main]
async fn main() {
    let mut args = Cli::parse();

    if args.no_color {
        colored::control::set_override(false);
    }

    // acquire the user's contact if not provided
    if args.username.is_none() {
        println!("Email: ");
        let mut email = String::new();
        std::io::stdin()
            .read_line(&mut email)
            .expect("Failed to read username.");
        args.username = Some(email.trim().into());
    }

    // handle log-in using OTP
    let client = perform_otp_login(&args.host, &args.username.clone().unwrap()).await;

    let state = AppState::new(args.host, args.username.unwrap(), client);
    run(Arc::new(RwLock::new(state))).await.unwrap();
}
