// TODO: see the README.md file for additional specifications
//  1. verify that cookies are being sent
//  2. shell commands and REPL loop -- rustyline
//  3. before implementing any actual commands, add 
//      3.1. HELP 
//      3.2. STATUS
//      3.3. EXIT/QUIT
//      3.4. CLEAR
//  4. log-out and auto-check for refresh on HTTP req -> refresh tokens
//     --> not sure how to make this clean
//  5. websocket features -> to be done AFTER the server websocket implementation is complete
mod auth;
mod state;

use auth::AuthState;
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

async fn run(state: Arc<RwLock<AppState>>) {
    // TODO: rustyline?

    println!(
        "{}",
        "┌─────────────────────────────────────────┐\n\
         │  Middleground CLI v1.0.0                │\n\
         │  Backend Testing & Debugging Tool       │\n\
         └─────────────────────────────────────────┘"
            .bright_cyan()
    );
}

async fn perform_otp_login<'a>(auth_client: AuthState<'a>, contact: &str) -> (String, String, u64) {
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

        if let Ok(ret) = auth_client.verify_otp(contact, otp.trim()).await {
            println!("{} Verification successful!\n", "✓".green());
            return ret;
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

    let mut state = AppState::new(
        args.host.clone(),
        args.username.clone().unwrap(),
    );

    // handle log-in using OTP
    let auth_client = AuthState::new(state.http_client(), args.host);
    let (at, rt, ea) = perform_otp_login(auth_client, &args.username.unwrap()).await;
    state.set_auth(at, rt, ea);

    run(Arc::new(RwLock::new(state))).await;
}
