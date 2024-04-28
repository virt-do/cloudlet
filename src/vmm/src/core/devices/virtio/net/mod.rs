pub mod device;
mod queue_handler;
mod simple_handler;
pub mod tuntap;

use crate::core::devices::virtio;

use self::tuntap::tap;

const NET_DEVICE_ID: u32 = 1;
const VIRTIO_NET_HDR_SIZE: usize = 12;
const RXQ_INDEX: u16 = 0;
const TXQ_INDEX: u16 = 1;

const TUN_F_CSUM: ::std::os::raw::c_uint = 1;
const TUN_F_TSO4: ::std::os::raw::c_uint = 2;
const TUN_F_TSO6: ::std::os::raw::c_uint = 4;
const TUN_F_UFO: ::std::os::raw::c_uint = 16;

#[derive(Debug)]
pub enum Error {
    Virtio(virtio::Error),
    Tap(tap::Error),
}

type Result<T> = std::result::Result<T, Error>;
