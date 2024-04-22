mod bindings;
pub mod device;
mod simple_handler;

use crate::core::network::tap;

const NET_DEVICE_ID: u32 = 1;
const VIRTIO_NET_HDR_SIZE: usize = 12;
const RXQ_INDEX: u16 = 0;
const TXQ_INDEX: u16 = 1;

#[derive(Debug)]
pub enum Error {
    Virtio(crate::core::devices::virtio::Error),
    Tap(tap::Error),
}
