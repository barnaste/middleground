/// Command parser for interactive shell
pub enum ShellCommand {
    // global commands
    Help,
    Status,
    Clear,
    Exit,

    // websocket commands
    WsConnect(Option<String>),
    WsDisconnect,
    WsStatus,

    // messaging commands
    Send(String),
    Reply(String, String),
    Edit(String, String),
    Delete(String),
    Subscribe(String),
    Unsubscribe,
    Messages(Option<usize>),

    // unknown
    Unknown(String),
}

impl ShellCommand {
    pub fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return ShellCommand::Unknown(String::new());
        }

        match parts[0] {
            "help" => ShellCommand::Help,
            "status" => ShellCommand::Status,
            "clear" => ShellCommand::Clear,
            "exit" => ShellCommand::Exit,
            "quit" => ShellCommand::Exit,

            _ => ShellCommand::Unknown(input.to_string()),
        }
    }
}
