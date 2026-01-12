use uuid::Uuid;

/// Command parser for interactive shell
pub enum ShellCommand {
    // global commands
    Help,
    Status,
    Clear,
    Exit,

    // websocket commands
    WsConnect(Uuid),
    WsDisconnect,

    // messaging commands
    Send(String),
    Reply(Uuid, String),
    Edit(Uuid, String),
    Delete(Uuid),
    Messages(Option<usize>),

    // unknown
    Unknown(String),
}

impl ShellCommand {
    pub fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let unknown_cmd = ShellCommand::Unknown(format!("Unknown command: '{}'", input));
        let invalid_id = |s: &str| ShellCommand::Unknown(format!("Invalid UUID format for {}", s));

        if parts.is_empty() {
            return unknown_cmd;
        }

        match parts[0] {
            "help" => ShellCommand::Help,
            "status" => ShellCommand::Status,
            "clear" => ShellCommand::Clear,
            "exit" | "quit" => ShellCommand::Exit,

            // WebSocket commands
            "ws" => {
                if parts.len() < 2 {
                    return unknown_cmd;
                }

                match parts[1] {
                    "connect" => {
                        let unknown_ws =
                            ShellCommand::Unknown("Usage: ws connect <channel>".to_string());
                        if parts.len() < 3 {
                            return unknown_ws;
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

            "messages" => {
                let limit = parts.get(1).and_then(|s| s.parse().ok());
                ShellCommand::Messages(limit)
            }

            _ => unknown_cmd,
        }
    }
}
