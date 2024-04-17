pub mod messages;

use std::io::{Read, Write};

use messages::{MessageType, Payload};
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
    MessageTypeDeserializationError(&'static str),
    PayloadDeserializationError(serde_json::Error),
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
    pub payload: Payload,
}

pub fn create_checksum(payload: &Payload) -> u16 {
    let mut checksum: u16 = 0;
    let json_payload = serde_json::to_string(payload).unwrap();
    for byte in json_payload.as_bytes() {
        checksum = checksum.wrapping_add(*byte as u16);
    }
    checksum
}

impl CloudletMessage {
    pub fn new(message_type: MessageType, payload: Payload) -> CloudletMessage {
        let checksum = create_checksum(&payload);

        CloudletMessage {
            message_type,
            checksum,
            payload,
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
        let message_type = message.message_type as u16;
        buffer.push((message.checksum >> 8) as u8);
        buffer.push((message.checksum & 0xFF) as u8);
        buffer.push((message_type >> 8) as u8);
        buffer.push((message_type & 0xFF) as u8);
        let json_payload = serde_json::to_string(&message.payload).unwrap();
        buffer.extend(json_payload.as_bytes());

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
        let message_type =
            MessageType::try_from(message_type).map_err(Error::MessageTypeDeserializationError)?;
        let json_payload = String::from_utf8_lossy(&buffer[4..]).into_owned();

        let payload =
            match message_type {
                MessageType::Start => serde_json::from_str(&json_payload)
                    .map_err(Error::PayloadDeserializationError)?,

                MessageType::Exit => serde_json::from_str(&json_payload)
                    .map_err(Error::PayloadDeserializationError)?,

                MessageType::Interrupt => serde_json::from_str(&json_payload)
                    .map_err(Error::PayloadDeserializationError)?,

                MessageType::Ok => serde_json::from_str(&json_payload)
                    .map_err(Error::PayloadDeserializationError)?,

                MessageType::Log => serde_json::from_str(&json_payload)
                    .map_err(Error::PayloadDeserializationError)?,
            };

        if checksum != create_checksum(&payload) {
            return Err(Error::ChecksumError);
        }

        Ok(CloudletMessage {
            message_type,
            checksum,
            payload,
        })
    }
}
