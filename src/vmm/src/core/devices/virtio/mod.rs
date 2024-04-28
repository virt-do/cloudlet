pub(crate) mod net;
mod register;

use event_manager::{
    Error as EvmgrError, MutEventSubscriber, RemoteEndpoint, Result as EvmgrResult, SubscriberId,
};
use kvm_ioctls::{IoEventAddress, VmFd};
use libc::EFD_NONBLOCK;
use std::{
    io,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};
use virtio_device::VirtioConfig;
use virtio_queue::{Queue, QueueT};
use vm_device::bus::{self, MmioAddress, MmioRange};
use vmm_sys_util::{errno, eventfd::EventFd};

// Device-independent virtio features.
mod features {
    pub const VIRTIO_F_RING_EVENT_IDX: u64 = 29;
    pub const VIRTIO_F_VERSION_1: u64 = 32;
    pub const VIRTIO_F_IN_ORDER: u64 = 35;
}

// This bit is set on the device interrupt status when notifying the driver about used
// queue events.
// TODO: There seem to be similar semantics when the PCI transport is used with MSI-X cap
// disabled. Let's figure out at some point if having MMIO as part of the name is necessary.
const VIRTIO_MMIO_INT_VRING: u8 = 0x01;

// The driver will write to the register at this offset in the MMIO region to notify the device
// about available queue events.
const VIRTIO_MMIO_QUEUE_NOTIFY_OFFSET: u64 = 0x50;

// TODO: Make configurable for each device maybe?
const QUEUE_MAX_SIZE: u16 = 256;

#[derive(Debug)]
pub enum Error {
    AlreadyActivated,
    BadFeatures(u64),
    Bus(bus::Error),
    Cmdline(linux_loader::cmdline::Error),
    Endpoint(EvmgrError),
    EventFd(io::Error),
    Overflow,
    QueuesNotValid,
    RegisterIoevent(errno::Error),
    RegisterIrqfd(errno::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
pub type Subscriber = Arc<Mutex<dyn MutEventSubscriber + Send>>;

#[derive(Copy, Clone)]
pub struct MmioConfig {
    pub range: MmioRange,
    // The interrupt assigned to the device.
    pub gsi: u32,
}

impl MmioConfig {
    pub fn new(base: u64, size: u64, gsi: u32) -> Result<Self> {
        MmioRange::new(MmioAddress(base), size)
            .map(|range| MmioConfig { range, gsi })
            .map_err(Error::Bus)
    }

    pub fn next(&self) -> Result<Self> {
        let range = self.range;
        let next_start = range
            .base()
            .0
            .checked_add(range.size())
            .ok_or(Error::Overflow)?;
        Self::new(next_start, range.size(), self.gsi + 1)
    }
}

struct Config<'a> {
    virtio: VirtioConfig<Queue>,
    mmio: MmioConfig,
    endpoint: RemoteEndpoint<Subscriber>,
    vm_fd: &'a VmFd,
    irqfd: Arc<EventFd>,
}

impl<'a> Config<'a> {
    pub fn new(
        virtio_cfg: VirtioConfig<Queue>,
        mmio: MmioConfig,
        endpoint: RemoteEndpoint<Subscriber>,
        vm_fd: &VmFd,
    ) -> Result<Self> {
        let irqfd = Arc::new(EventFd::new(EFD_NONBLOCK).map_err(Error::EventFd)?);

        Ok(Self {
            virtio: virtio_cfg,
            mmio,
            endpoint,
            vm_fd,
            irqfd,
        })
    }

    // Perform common initial steps for device activation based on the configuration, and return
    // a `Vec` that contains `EventFd`s registered as ioeventfds, which are used to convey queue
    // notifications coming from the driver.
    pub fn prepare_activate(&self) -> Result<Vec<EventFd>> {
        if self.virtio.queues.iter().all(|queue| !queue.ready()) {
            return Err(Error::QueuesNotValid);
        }

        if self.virtio.device_activated {
            return Err(Error::AlreadyActivated);
        }

        // We do not support legacy drivers.
        if self.virtio.driver_features & (1 << features::VIRTIO_F_VERSION_1) == 0 {
            return Err(Error::BadFeatures(self.virtio.driver_features));
        }

        let mut ioevents = Vec::new();

        // Right now, we operate under the assumption all queues are marked ready by the device
        // (which is true until we start supporting devices that can optionally make use of
        // additional queues on top of the defaults).
        for i in 0..self.virtio.queues.len() {
            let fd = EventFd::new(EFD_NONBLOCK).map_err(Error::EventFd)?;

            // Register the queue event fd.
            self.vm_fd
                .register_ioevent(
                    &fd,
                    &IoEventAddress::Mmio(
                        self.mmio.range.base().0 + VIRTIO_MMIO_QUEUE_NOTIFY_OFFSET,
                    ),
                    // The maximum number of queues should fit within an `u16` according to the
                    // standard, so the conversion below is always expected to succeed.
                    u32::try_from(i).unwrap(),
                )
                .map_err(Error::RegisterIoevent)?;

            ioevents.push(fd);
        }

        Ok(ioevents)
    }

    // Perform the final steps of device activation based on the inner configuration and the
    // provided subscriber that's going to handle the device queues. We'll extend this when
    // we start support devices that make use of multiple handlers (i.e. for multiple queues).
    pub fn finalize_activate(&mut self, handler: Subscriber) -> Result<()> {
        // Register the queue handler with the `EventManager`. We could record the `sub_id`
        // (and/or keep a handler clone) for further interaction (i.e. to remove the subscriber at
        // a later time, retrieve state, etc).
        let _sub_id = self
            .endpoint
            .call_blocking(move |mgr| -> EvmgrResult<SubscriberId> {
                Ok(mgr.add_subscriber(handler))
            })
            .map_err(Error::Endpoint)?;

        self.virtio.device_activated = true;

        Ok(())
    }
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
