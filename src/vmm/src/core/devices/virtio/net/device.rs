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
use vm_device::{bus::MmioAddress, MutDeviceMmio};
use vm_memory::{GuestAddress, GuestAddressSpace, GuestUsize};
use vmm_sys_util::eventfd::EventFd;

use crate::core::{
    devices::virtio::{register_mmio_device, SingleFdSignalQueue, QUEUE_MAX_SIZE},
    network::tap::Tap,
};

use super::{bindings, Error, NET_DEVICE_ID, VIRTIO_NET_HDR_SIZE};

struct NetConfig {
    virtio: VirtioConfig<Queue>,
    irqfd: Arc<EventFd>,
}

pub struct Net<M: GuestAddressSpace> {
    tap_name: String,
    vmmio_parameter: String,
    _mem: M,
    config: NetConfig,
}

impl<M> Net<M>
where
    M: GuestAddressSpace,
{
    pub fn new(
        tap_name: String,
        size: GuestUsize,
        baseaddr: GuestAddress,
        irq: u32,
        mem: M,
    ) -> Result<Self, linux_loader::cmdline::Error> {
        let device_features = (1 << VIRTIO_F_VERSION_1)
            // | (1 << VIRTIO_F_RING_EVENT_IDX)
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
            Queue::new(QUEUE_MAX_SIZE).unwrap(),
            Queue::new(QUEUE_MAX_SIZE).unwrap(),
        ];
        let virtio_cfg = VirtioConfig::new(device_features, queues, config_space);

        let vmmio_parameter = register_mmio_device(size, baseaddr, irq, None).unwrap();
        let config = NetConfig { virtio: virtio_cfg };

        Ok(Self {
            tap_name,
            vmmio_parameter,
            _mem: mem,
            config,
        })
    }

    pub fn get_vmmio_parameter(&self) -> String {
        self.vmmio_parameter.clone()
    }

    fn register_mmio_device() {}
}

impl<M: GuestAddressSpace> VirtioDeviceType for Net<M> {
    fn device_type(&self) -> u32 {
        NET_DEVICE_ID
    }
}

impl<M: GuestAddressSpace> Borrow<VirtioConfig<Queue>> for Net<M> {
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.config.virtio
    }
}

impl<M: GuestAddressSpace> BorrowMut<VirtioConfig<Queue>> for Net<M> {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.config.virtio
    }
}

impl<M: GuestAddressSpace> VirtioDeviceActions for Net<M> {
    type E = Error;

    fn activate(&mut self) -> Result<(), Error> {
        // Set offload flags to match the relevant virtio features of the device (for now,
        // statically set in the constructor.
        let tap = Tap::new(2).map_err(Error::Tap)?;

        // The layout of the header is specified in the standard and is 12 bytes in size. We
        // should define this somewhere.
        tap.set_vnet_hdr_size(VIRTIO_NET_HDR_SIZE as i32)
            .map_err(Error::Tap)?;

        let driver_notify = SingleFdSignalQueue {
            irqfd: self.config.irqfd.clone(),
            interrupt_status: self.config.virtio.interrupt_status.clone(),
        };

        let mut ioevents = self.config.prepare_activate().map_err(Error::Virtio)?;

        let rxq = self.config.virtio.queues.remove(0);
        let txq = self.config.virtio.queues.remove(0);
        let inner = SingleHandler::new(driver_notify, rxq, txq, tap);

        let handler = Arc::new(Mutex::new(QueueHandler {
            inner,
            rx_ioevent: ioevents.remove(0),
            tx_ioevent: ioevents.remove(0),
        }));

        self.cfg.finalize_activate(handler).map_err(Error::Virtio)
    }

    fn reset(&mut self) -> std::result::Result<(), Error> {
        // Not implemented for now.
        Ok(())
    }
}

impl<M: GuestAddressSpace> VirtioMmioDevice<M> for Net<M> {}

impl<M: GuestAddressSpace> MutDeviceMmio for Net<M> {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
