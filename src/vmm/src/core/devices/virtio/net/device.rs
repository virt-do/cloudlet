use super::queue_handler::QueueHandler;
use super::{
    simple_handler::SimpleHandler, tuntap::tap::Tap, Error, Result, NET_DEVICE_ID,
    VIRTIO_NET_HDR_SIZE,
};
use crate::core::devices::virtio::features::VIRTIO_F_RING_EVENT_IDX;
use crate::core::devices::virtio::net::tuntap::open_tap::open_tap;
use crate::core::devices::virtio::register::register_mmio_device;
use crate::core::devices::virtio::{
    self, Config, MmioConfig, SingleFdSignalQueue, Subscriber, QUEUE_MAX_SIZE,
};
use event_manager::RemoteEndpoint;
use kvm_ioctls::VmFd;
use std::net::Ipv4Addr;
use std::{
    borrow::{Borrow, BorrowMut},
    sync::{Arc, Mutex},
};
use virtio_bindings::{
    virtio_config::{VIRTIO_F_IN_ORDER, VIRTIO_F_VERSION_1},
    virtio_net::{
        VIRTIO_NET_F_CSUM, VIRTIO_NET_F_GUEST_CSUM, VIRTIO_NET_F_GUEST_TSO4,
        VIRTIO_NET_F_GUEST_TSO6, VIRTIO_NET_F_GUEST_UFO, VIRTIO_NET_F_HOST_TSO4,
        VIRTIO_NET_F_HOST_TSO6, VIRTIO_NET_F_HOST_UFO,
    },
};
use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};
use virtio_queue::{Queue, QueueT};
use vm_device::device_manager::IoManager;
use vm_device::{bus::MmioAddress, MutDeviceMmio};
use vm_memory::GuestMemoryMmap;

pub struct Net {
    mem: Arc<GuestMemoryMmap>,
    pub config: Config,
    tap: Arc<Mutex<Tap>>,
}

impl Net {
    pub fn new(
        mem: Arc<GuestMemoryMmap>,
        device_mgr: Arc<Mutex<IoManager>>,
        mmio_cfg: MmioConfig,
        ip_addr: Ipv4Addr,
        mask: Ipv4Addr,
        irq: u32,
        endpoint: RemoteEndpoint<Subscriber>,
        vm_fd: Arc<VmFd>,
        cmdline_extra_parameters: &mut Vec<String>,
    ) -> Result<Arc<Mutex<Self>>> {
        let device_features = (1 << VIRTIO_F_VERSION_1)
            | (1 << VIRTIO_F_RING_EVENT_IDX)
            | (1 << VIRTIO_F_IN_ORDER)
            | (1 << VIRTIO_NET_F_CSUM)
            | (1 << VIRTIO_NET_F_GUEST_CSUM)
            | (1 << VIRTIO_NET_F_GUEST_TSO4)
            | (1 << VIRTIO_NET_F_GUEST_TSO6)
            | (1 << VIRTIO_NET_F_GUEST_UFO)
            | (1 << VIRTIO_NET_F_HOST_TSO4)
            | (1 << VIRTIO_NET_F_HOST_TSO6)
            | (1 << VIRTIO_NET_F_HOST_UFO);

        let config_space = Vec::new();
        let queues = vec![
            Queue::new(QUEUE_MAX_SIZE).map_err(|_| Error::Virtio(virtio::Error::QueuesNotValid))?,
            Queue::new(QUEUE_MAX_SIZE).map_err(|_| Error::Virtio(virtio::Error::QueuesNotValid))?,
        ];

        let virtio_cfg = VirtioConfig::new(device_features, queues, config_space);

        let cfg = Config::new(virtio_cfg, mmio_cfg, endpoint, vm_fd).map_err(Error::Virtio)?;

        // Set offload flags to match the relevant virtio features of the device (for now,
        // statically set in the constructor.
        let tap = open_tap(None, Some(ip_addr), Some(mask), &mut None, None, None)
            .map_err(Error::TunTap)?;

        // The layout of the header is specified in the standard and is 12 bytes in size. We
        // should define this somewhere.
        tap.set_vnet_hdr_size(VIRTIO_NET_HDR_SIZE as i32)
            .map_err(Error::Tap)?;

        let net = Arc::new(Mutex::new(Net {
            mem,
            config: cfg,
            tap: Arc::new(Mutex::new(tap)),
        }));

        let param = register_mmio_device(mmio_cfg, device_mgr, irq, None, net.clone())
            .map_err(Error::Virtio)?;
        cmdline_extra_parameters.push(param);

        Ok(net)
    }
}

impl VirtioDeviceType for Net {
    fn device_type(&self) -> u32 {
        NET_DEVICE_ID
    }
}

impl Borrow<VirtioConfig<Queue>> for Net {
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.config.virtio
    }
}

impl BorrowMut<VirtioConfig<Queue>> for Net {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.config.virtio
    }
}

impl VirtioDeviceActions for Net {
    type E = Error;

    fn activate(&mut self) -> Result<()> {
        let driver_notify = SingleFdSignalQueue {
            irqfd: self.config.irqfd.clone(),
            interrupt_status: self.config.virtio.interrupt_status.clone(),
        };

        let mut ioevents = self.config.prepare_activate().map_err(Error::Virtio)?;

        let rxq = self.config.virtio.queues.remove(0);
        let txq = self.config.virtio.queues.remove(0);
        let inner = SimpleHandler::new(driver_notify, rxq, txq, self.tap.clone(), self.mem.clone());

        let handler = Arc::new(Mutex::new(QueueHandler {
            inner,
            rx_ioevent: ioevents.remove(0),
            tx_ioevent: ioevents.remove(0),
        }));

        self.config
            .finalize_activate(handler)
            .map_err(Error::Virtio)
    }

    fn reset(&mut self) -> std::result::Result<(), Error> {
        // Not implemented for now.
        Ok(())
    }
}

impl VirtioMmioDevice for Net {}

impl MutDeviceMmio for Net {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
