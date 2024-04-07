mod messages;

use std::io::{Read, Write};

use messages::MessageType;
use serialport::TTYPort;

const TERMINATOR: u8 = 0xC0;
const ESCAPE: u8 = 0xDB;
const ESCAPE_TERMINATOR: u8 = 0xDC;
const ESCAPE_ESCAPE: u8 = 0xDD;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    ChecksumError,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct CloudletProtocol {
    pub serial_port: TTYPort,
}

#[derive(Debug)]
pub struct CloudletMessage {
    pub message_type: MessageType,
    pub checksum: u16,
    pub payload: String,
}

pub fn create_checksum(content: &str) -> u16 {
    let mut checksum: u16 = 0;
    for byte in content.as_bytes() {
        checksum = checksum.wrapping_add(*byte as u16);
    }
    checksum
}

impl CloudletMessage {
    pub fn new(message_type: MessageType, content: String) -> CloudletMessage {
        let checksum = create_checksum(&content);

        CloudletMessage {
            message_type,
            checksum,
            payload: content,
        }
    }
}

impl CloudletProtocol {
    pub fn new(serial_port: TTYPort) -> CloudletProtocol {
        CloudletProtocol { serial_port }
    }

    /// Escape a buffer to avoid the terminator byte and the escape byte
    fn escape_buffer(buffer: Vec<u8>) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for byte in buffer {
            match byte {
                TERMINATOR => {
                    result.push(ESCAPE);
                    result.push(ESCAPE_TERMINATOR);
                }
                ESCAPE => {
                    result.push(ESCAPE);
                    result.push(ESCAPE_ESCAPE);
                }
                _ => {
                    result.push(byte);
                }
            }
        }
        result
    }

    /// Write a message to the serial port
    /// Escape the message to avoid the terminator byte
    /// and the escape byte
    /// The message is terminated with the terminator byte
    /// The message is formatted as follows:
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    pub fn send_message(&mut self, message: CloudletMessage) {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.push((message.checksum >> 8) as u8);
        buffer.push((message.checksum & 0xFF) as u8);
        buffer.push((message.message_type as u16 >> 8) as u8);
        buffer.extend(message.payload.as_bytes());

        buffer = CloudletProtocol::escape_buffer(buffer);

        buffer.push(TERMINATOR);

        self.serial_port
            .write_all(&buffer)
            .expect("Failed to write message to serial port");
    }

    /// Read a message from the serial port
    pub fn read_message(&mut self) -> Result<CloudletMessage> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut byte = [0];

        loop {
            match self.serial_port.read_exact(&mut byte) {
                Ok(_) => match byte[0] {
                    TERMINATOR => break,
                    ESCAPE => match self.serial_port.read_exact(&mut byte) {
                        Ok(_) => match byte[0] {
                            ESCAPE_TERMINATOR => buffer.push(TERMINATOR),
                            ESCAPE_ESCAPE => buffer.push(ESCAPE),
                            _ => {
                                return Err(Error::IoError(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Invalid escape sequence",
                                )))
                            }
                        },
                        Err(e) => return Err(Error::IoError(e)),
                    },
                    _ => buffer.push(byte[0]),
                },
                Err(e) => return Err(Error::IoError(e)),
            }
        }

        if buffer.len() < 4 {
            return Err(Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Message too short",
            )));
        }

        let checksum = u16::from_be_bytes([buffer[0], buffer[1]]);
        let message_type = u16::from_be_bytes([buffer[2], buffer[3]]);
        let message_type = MessageType::from(message_type);
        let content = String::from_utf8_lossy(&buffer[4..]).into_owned();

        if checksum != create_checksum(&content) {
            return Err(Error::ChecksumError);
        }

        Ok(CloudletMessage {
            message_type,
            checksum,
            payload: content,
        })
    }
}
