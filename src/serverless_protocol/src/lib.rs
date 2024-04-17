pub mod messages;

use std::io::{Read, Write};

use messages::{MessageType, Payload};
use serialport::TTYPort;

use sha256::digest;

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
    pub checksum: Vec<u8>,
    pub payload: Payload,
}

pub fn create_checksum(message_type: &MessageType, payload: &Payload) -> Vec<u8> {
    let type_bytes = message_type.to_owned() as u8;
    let mut bytes = vec![type_bytes];

    bytes.append(&mut bincode::serialize(payload).unwrap());

    digest(&bytes).as_bytes().to_vec()
}

impl CloudletMessage {
    pub fn new(message_type: MessageType, payload: Payload) -> CloudletMessage {
        let checksum = create_checksum(&message_type, &payload);

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
        buffer.append(&mut message.checksum.clone());
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

        let checksum = buffer[0..64].to_vec();
        let message_type = u16::from_be_bytes([buffer[64], buffer[65]]);
        let message_type =
            MessageType::try_from(message_type).map_err(Error::MessageTypeDeserializationError)?;
        let json_payload = String::from_utf8_lossy(&buffer[66..]).into_owned();

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

        if checksum != create_checksum(&message_type, &payload) {
            return Err(Error::ChecksumError);
        }

        Ok(CloudletMessage {
            message_type,
            checksum,
            payload,
        })
    }
}
