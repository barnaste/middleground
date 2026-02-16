//! Command parser for the interactive shell.
//!
//! Parses user input into structured commands that can be executed by the main application loop.

use uuid::Uuid;

/// Parsed shell commands from user input.
///
/// Each variant represents a specific action the user can take.
pub enum ShellCommand {
    // === Global Commands ===
    /// Display help information
    Help,

    /// Show connection status
    Status,

    /// Clear the terminal screen
    Clear,

    /// Exit the application
    Exit,

    // === WebSocket Commands ===
    /// Connect to a conversation channel
    WsConnect(Uuid),

    /// Disconnect from the current channel
    WsDisconnect,

    // === Messaging Commands ===
    /// Send a new message
    Send(String),

    /// Reply to an existing message
    Reply(Uuid, String),

    /// Edit an existing message
    Edit(Uuid, String),

    /// Delete an existing message
    Delete(Uuid),

    // Unknown or invalid command
    Unknown(String),
}

impl ShellCommand {
    /// Parse a line of user input into a command.
    ///
    /// Supports whitespace-separated arguments and validates UUIDs.
    /// Unknown commands are captured in the `Unknown` variant with an error message.
    ///
    /// # Arguments
    /// * `input` - Raw user input string
    ///
    /// # Returns
    ///
    /// A parsed `ShellCommand`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use shellcmd::ShellCommand;
    ///
    /// let cmd = ShellCommand::parse("send hello world");
    /// let cmd = ShellCommand::parse("ws connect 550e8400-e29b-41b4-a716-446655440000");
    /// ```
    pub fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Self::Unknown(String::new());
        }

        let unknown_cmd = ShellCommand::Unknown(format!("Unknown command: '{}'", input));
        let invalid_id = |s: &str| ShellCommand::Unknown(format!("Invalid UUID format for {}", s));

        match parts[0] {
            // === Global Commands ===
            "help" => ShellCommand::Help,
            "status" => ShellCommand::Status,
            "clear" => ShellCommand::Clear,
            "exit" | "quit" => ShellCommand::Exit,

            // === WebSocket Commands ===
            "ws" => {
                if parts.len() < 2 {
                    return unknown_cmd;
                }

                match parts[1] {
                    "connect" => {
                        if parts.len() < 3 {
                            return ShellCommand::Unknown(
                                "Usage: ws connect <channel>".to_string(),
                            );
                        }

                        match Uuid::parse_str(parts[2]) {
                            Ok(id) => ShellCommand::WsConnect(id),
                            Err(_) => invalid_id("channel"),
                        }
                    }
                    "disconnect" => ShellCommand::WsDisconnect,
                    "status" => ShellCommand::Status,
                    _ => unknown_cmd,
                }
            }

            // === Messaging Commands ===
            "send" | "s" => {
                if parts.len() < 2 {
                    return ShellCommand::Unknown("Usage: send <msg>".to_string());
                }

                ShellCommand::Send(parts[1..].join(" "))
            }

            "reply" | "r" => {
                if parts.len() < 3 {
                    return ShellCommand::Unknown("Usage: reply <msg_id> <msg>".to_string());
                }

                match Uuid::parse_str(parts[1]) {
                    Ok(id) => ShellCommand::Reply(id, parts[2..].join(" ")),
                    Err(_) => invalid_id("msg_id"),
                }
            }

            "edit" | "e" => {
                if parts.len() < 3 {
                    return ShellCommand::Unknown("Usage: edit <msg_id> <msg>".to_string());
                }

                match Uuid::parse_str(parts[1]) {
                    Ok(id) => ShellCommand::Edit(id, parts[2..].join(" ")),
                    Err(_) => invalid_id("msg_id"),
                }
            }

            "delete" | "d" => {
                if parts.len() < 2 {
                    return ShellCommand::Unknown("Usage: delete <msg_id>".to_string());
                }

                match Uuid::parse_str(parts[1]) {
                    Ok(id) => ShellCommand::Delete(id),
                    Err(_) => invalid_id("msg_id"),
                }
            }

            _ => unknown_cmd,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_send() {
        match ShellCommand::parse("send hello world") {
            ShellCommand::Send(content) => assert_eq!(content, "hello world"),
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_parse_help() {
        match ShellCommand::parse("help") {
            ShellCommand::Help => {}
            _ => panic!("Expected Help command"),
        }
    }

    #[test]
    fn test_parse_unknown() {
        match ShellCommand::parse("invalid command") {
            ShellCommand::Unknown(_) => {}
            _ => panic!("Expected Unknown command"),
        }
    }
}
