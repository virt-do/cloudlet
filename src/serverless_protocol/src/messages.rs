#[derive(Debug, Clone)]
pub enum MessageType {
    Start = 0,
    Exit = 1,
    Interrupt = 2,
    Ok = 3,
    Log = 4,
}

impl From<u16> for MessageType {
    fn from(value: u16) -> Self {
        match value {
            0 => MessageType::Start,
            1 => MessageType::Exit,
            2 => MessageType::Interrupt,
            3 => MessageType::Ok,
            4 => MessageType::Log,
            // TODO: Handle error
            _ => panic!("Invalid message type"),
        }
    }
}

/// Expects OkMessage
#[derive(Debug, Clone)]
pub struct StartMessage {
    content: String,
}

/// Expects OkMessage
#[derive(Debug, Clone)]
pub struct ExitMessage {
    code: i32,
    stdin: String,
    stdout: String,
}

/// Expects OkMessage
#[derive(Debug, Clone)]
pub struct InterruptMessage {
    signal: i32,
}

/// Mostly used to answer other messages
#[derive(Debug, Clone)]
pub struct OkMessage {}

/// Expects OkMessage
#[derive(Debug, Clone)]
pub struct LogMessage {
    kind: String,
    content: String,
}
