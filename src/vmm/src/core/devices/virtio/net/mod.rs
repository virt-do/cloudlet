mod bridge;
pub mod device;
pub mod iptables;
mod queue_handler;
mod simple_handler;
pub mod tuntap;

use crate::core::devices::virtio;

use self::tuntap::{open_tap, tap};

const NET_DEVICE_ID: u32 = 1;
const VIRTIO_NET_HDR_SIZE: usize = 12;
const RXQ_INDEX: u16 = 0;
const TXQ_INDEX: u16 = 1;
const BRIDGE_NAME: &str = "br0";

#[derive(Debug)]
pub enum Error {
    Virtio(virtio::Error),
    TunTap(open_tap::Error),
    Tap(tap::Error),
    Bridge(bridge::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn xx_netmask_width<const SZ: usize>(netmask: [u8; SZ]) -> u8 {
    netmask.iter().map(|x| x.count_ones() as u8).sum()
}
