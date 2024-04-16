extern crate clap;
extern crate serverless_protocol;

use std::{thread, time::Duration};

use clap::Parser;
use serverless_protocol::{
    messages::{MessageType, Payload, StartMessage},
    CloudletMessage, CloudletProtocol,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    serial_path_a: String,

    #[arg(long)]
    serial_path_b: String,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

    let handle_a = thread::spawn(|| {
        let serial_path = args.serial_path_a;

        let serial_port = serialport::new(serial_path, 115_200)
            .timeout(Duration::from_secs(10))
            .open_native()
            .expect("Failed to open serial port");

        let mut protocol = CloudletProtocol::new(serial_port);

        println!("waiting for message");

        let message = protocol
            .read_message()
            .expect("Failed to read message from serial port");

        println!("{:?}", message);
    });

    let handle_b = thread::spawn(|| {
        let serial_path = args.serial_path_b;

        let serial_port = serialport::new(serial_path, 115_200)
            .timeout(Duration::from_secs(10))
            .open_native()
            .expect("Failed to open serial port");

        let mut protocol = CloudletProtocol::new(serial_port);

        let message = CloudletMessage::new(
            MessageType::Start,
            Payload::Start(StartMessage::new("Hello, World!".to_string())),
        );

        println!("sending message: {:?}", message);

        protocol.send_message(message);

        println!("message sent")
    });

    handles.push(handle_a);
    handles.push(handle_b);

    for handle in handles {
        handle.join().unwrap();
    }
}
