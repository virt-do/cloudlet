use linux_loader::cmdline;
use std::{
    io, result,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};
use virtio_bindings::virtio_mmio::VIRTIO_MMIO_INT_VRING;
use vm_device::bus;
use vm_memory::{Address, GuestAddress, GuestUsize};
use vmm_sys_util::{errno, eventfd::EventFd};

pub mod net;

pub type Result<T> = result::Result<T, Error>;

const QUEUE_MAX_SIZE: u16 = 256;

#[derive(Debug)]
pub enum Error {
    AlreadyActivated,
    BadFeatures(u64),
    Bus(bus::Error),
    Cmdline(linux_loader::cmdline::Error),
    // Endpoint(EvmgrError),
    EventFd(io::Error),
    Overflow,
    QueuesNotValid,
    RegisterIoevent(errno::Error),
    RegisterIrqfd(errno::Error),
}

pub fn register_mmio_device(
    size: GuestUsize,
    baseaddr: GuestAddress,
    irq: u32,
    id: Option<u32>,
) -> Result<String> {
    // !TODO Register to MmioManager

    // Pass to kernel command line
    if size == 0 {
        return Err(cmdline::Error::MmioSize);
    }

    let mut device_str = format!(
        "virtio_mmio.device={}@0x{:x?}:{}",
        guestusize_to_str(size),
        baseaddr.raw_value(),
        irq
    );
    if let Some(id) = id {
        device_str.push_str(format!(":{}", id).as_str());
    }
    Ok(device_str)
}

fn guestusize_to_str(size: GuestUsize) -> String {
    const KB_MULT: u64 = 1 << 10;
    const MB_MULT: u64 = KB_MULT << 10;
    const GB_MULT: u64 = MB_MULT << 10;

    if size % GB_MULT == 0 {
        return format!("{}G", size / GB_MULT);
    }
    if size % MB_MULT == 0 {
        return format!("{}M", size / MB_MULT);
    }
    if size % KB_MULT == 0 {
        return format!("{}K", size / KB_MULT);
    }
    size.to_string()
}

/// Simple trait to model the operation of signalling the driver about used events
/// for the specified queue.
// TODO: Does this need renaming to be relevant for packed queues as well?
pub trait SignalUsedQueue {
    // TODO: Should this return an error? This failing is not really recoverable at the interface
    // level so the expectation is the implementation handles that transparently somehow.
    fn signal_used_queue(&self, index: u16);
}

/// Uses a single irqfd as the basis of signalling any queue (useful for the MMIO transport,
/// where a single interrupt is shared for everything).
pub struct SingleFdSignalQueue {
    pub irqfd: Arc<EventFd>,
    pub interrupt_status: Arc<AtomicU8>,
}

impl SignalUsedQueue for SingleFdSignalQueue {
    fn signal_used_queue(&self, _index: u16) {
        self.interrupt_status
            .fetch_or(VIRTIO_MMIO_INT_VRING, Ordering::SeqCst);
        self.irqfd
            .write(1)
            .expect("Failed write to eventfd when signalling queue");
    }
}
