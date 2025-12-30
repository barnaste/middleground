/// Command parser for interactive shell
enum ShellCommand {
    // global commands
    Help(Option<String>),
    Status,
    Clear,
    Exit,

    // websocket commands
    WsConnect(Option<String>),
    WsDisconnect,
    WsStatus,

    // messaging commands
    Send(String),
    Subscribe(String),
    Unsubscribe,
    Messages(Option<usize>),

    // unknown
    Unknown(String),
}

impl ShellCommand {
    fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        todo!()
    }
}
