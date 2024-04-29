pub mod device;
mod queue_handler;
mod simple_handler;
pub mod tuntap;

use crate::core::devices::virtio;

use self::tuntap::{open_tap, tap};

const NET_DEVICE_ID: u32 = 1;
const VIRTIO_NET_HDR_SIZE: usize = 12;
const RXQ_INDEX: u16 = 0;
const TXQ_INDEX: u16 = 1;

#[derive(Debug)]
pub enum Error {
    Virtio(virtio::Error),
    TunTap(open_tap::Error),
    Tap(tap::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
