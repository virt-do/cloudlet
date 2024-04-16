use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum MessageType {
    Start = 0,
    Exit = 1,
    Interrupt = 2,
    Ok = 3,
    Log = 4,
}

impl TryFrom<u16> for MessageType {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::Start),
            1 => Ok(MessageType::Exit),
            2 => Ok(MessageType::Interrupt),
            3 => Ok(MessageType::Ok),
            4 => Ok(MessageType::Log),
            _ => Err("Invalid message type"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    Start(StartMessage),
    Exit(ExitMessage),
    Interrupt(InterruptMessage),
    Ok(OkMessage),
    Log(LogMessage),
}

/// Expects OkMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMessage {
    content: String,
}

impl StartMessage {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

/// Expects OkMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitMessage {
    code: i32,
    stderr: String,
    stdout: String,
}

impl ExitMessage {
    pub fn new(code: i32, stderr: String, stdout: String) -> Self {
        Self {
            code,
            stderr,
            stdout,
        }
    }
}

/// Expects OkMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptMessage {
    signal: i32,
}

impl InterruptMessage {
    pub fn new(signal: i32) -> Self {
        Self { signal }
    }
}

/// Mostly used to answer other messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkMessage {}

impl OkMessage {
    pub fn new() -> Self {
        Self {}
    }
}

/// Expects OkMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    kind: String,
    content: String,
}

impl LogMessage {
    pub fn new(kind: String, content: String) -> Self {
        Self { kind, content }
    }
}
