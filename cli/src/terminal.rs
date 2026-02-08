//! Terminal utilities for managing prompt display and async output
//!
//! This module provides utilities for coordinating terminal output between the interactive REPL
//! and asynchronous messages, ensuring the prompt is always properly displayed and messages don't
//! corrupt the input line.

use std::{io::{self, Write}, sync::Arc};
use tokio::sync::Mutex;

/// ANSI escape sequences for terminal control
pub mod ansi {
    /// clear from cursor to end of line
    pub const CLEAR_LINE: &str = "\x1B[K";
    /// move cursor to column 0
    pub const CURSOR_COL_0: &str = "\r";
    /// clear entire screen
    pub const CLEAR_SCREEN: &str = "\x1B[2J";
    /// move cursor to home position (top-left corner)
    pub const CURSOR_HOME: &str = "\x1B[H";
}

/// Manages terminal output to coordinate between REPL and async messages
///
/// This manager ensures that asynchronous messages (e.g. from websockets) don't corrupt the
/// command prompt by clearing the prompt line, printing the message, and redrawing the prompt.
#[derive(Clone)]
pub struct TerminalManager {
    current_prompt: Arc<Mutex<Option<String>>>,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new() -> Self {
        Self {
            current_prompt: Arc::new(Mutex::new(None)),
        }
    }

    /// Update the current prompt text
    ///
    /// This should be called whenever a prompt is sent and is pending user input.
    pub async fn set_prompt(&self, prompt: String) {
        *self.current_prompt.lock().await = Some(prompt);
    }

    /// Remove the current prompt text
    ///
    /// This should be called once the prompt has received user input.
    pub async fn unset_prompt(&self) {
        *self.current_prompt.lock().await = None;
    }

    /// Print a message, handling prompt redrawing if necessary
    ///
    /// If currently at a prompt, this will:
    /// 1. Clear the prompt line
    /// 2. Print the message
    /// 3. Redraw the prompt
    ///
    /// If not at a prompt, prints normally
    pub async fn print_message(&self, message: &str) {
        let prompt = self.current_prompt.lock().await;
        if let Some(ref prompt) = *prompt {
            // we should clear the prompt first, by erasing the current line 
            // containing the prompt, then printing it after the message
            print!("{}{}", ansi::CURSOR_COL_0, ansi::CLEAR_LINE);
            println!("{}", message);
            print!("{}", prompt);

            io::stdout().flush().unwrap();
        } else {
            println!("{}", message);
        }
    }
}
