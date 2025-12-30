// TODO: see the README.md file for additional specifications
//  1. before implementing any actual commands, add
//      3.1. HELP
//      3.2. STATUS
//      3.3. EXIT/QUIT (SHOULD LOGOUT)
//      3.4. CLEAR
//  2. websocket features -> to be done AFTER the server websocket implementation is complete
mod auth;
mod shellcmd;
mod state;

use anyhow::Result;
use auth::AuthClient;
use clap::Parser;
use colored::Colorize;
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

        match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("{} {:?}", "✗".red(), err);
            }
        }
    }

    println!("\n{} Goodbye!", "✓".green());
    Ok(())
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
    // let client = perform_otp_login(&args.host, &args.username.clone().unwrap()).await;
    let client = AuthClient::new(&args.host);

    let state = AppState::new(args.host, args.username.unwrap(), client);
    run(Arc::new(RwLock::new(state))).await.unwrap();
}
