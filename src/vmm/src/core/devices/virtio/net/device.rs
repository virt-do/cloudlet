use super::queue_handler::QueueHandler;
use super::Result;
use super::{
    simple_handler::SimpleHandler, tuntap::tap::Tap, Error, NET_DEVICE_ID, VIRTIO_NET_HDR_SIZE,
};
use crate::core::devices::virtio::register::register_mmio_device;
use crate::core::devices::virtio::{
    Config, MmioConfig, SingleFdSignalQueue, Subscriber, QUEUE_MAX_SIZE,
};
use event_manager::RemoteEndpoint;
use kvm_ioctls::VmFd;
use std::ops::Deref;
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
use vm_memory::{GuestAddressSpace, GuestMemory};

pub struct Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Sync,
    <M as Deref>::Target: GuestMemory,
{
    mem: Arc<M>,
    config: Config<'a>,
    tap_name: String,
    vmmio_parameter: String,
}

impl<'a, M> Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Send + 'static + Sync,
    <M as Deref>::Target: GuestMemory,
{
    pub fn new(
        mem: Arc<M>,
        mmio_cfg: MmioConfig,
        tap_name: String,
        irq: u32,
        endpoint: RemoteEndpoint<Subscriber>,
        vm_fd: &VmFd,
    ) -> Result<Arc<Mutex<Self>>> {
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

        // let vmmio_parameter = register_mmio_device(size, baseaddr, irq, None).unwrap();
        let cfg = Config::new(virtio_cfg, mmio_cfg, endpoint, vm_fd).unwrap();

        let net = Arc::new(Mutex::new(Net {
            mem,
            config: cfg,
            tap_name,
            vmmio_parameter: "".to_string(),
        }));

        register_mmio_device(mmio_cfg, irq, None);
        // env.register_mmio_device(net.clone())
        // .map_err(Error::Virtio)?;

        Ok(net)
    }

    pub fn get_vmmio_parameter(&self) -> String {
        self.vmmio_parameter.clone()
    }
}

impl<'a, M> VirtioDeviceType for Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Send + 'static + Sync,
    <M as Deref>::Target: GuestMemory,
{
    fn device_type(&self) -> u32 {
        NET_DEVICE_ID
    }
}

impl<'a, M> Borrow<VirtioConfig<Queue>> for Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Send + 'static + Sync,
    <M as Deref>::Target: GuestMemory,
{
    fn borrow(&self) -> &VirtioConfig<Queue> {
        &self.config.virtio
    }
}

impl<'a, M> BorrowMut<VirtioConfig<Queue>> for Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Send + 'static + Sync,
    <M as Deref>::Target: GuestMemory,
{
    fn borrow_mut(&mut self) -> &mut VirtioConfig<Queue> {
        &mut self.config.virtio
    }
}

impl<'a, M> VirtioDeviceActions for Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Send + 'static + Sync,
    <M as Deref>::Target: GuestMemory,
{
    type E = Error;

    fn activate(&mut self) -> Result<()> {
        // Set offload flags to match the relevant virtio features of the device (for now,
        // statically set in the constructor.
        let tap = Tap::new(2).unwrap();

        // The layout of the header is specified in the standard and is 12 bytes in size. We
        // should define this somewhere.
        tap.set_vnet_hdr_size(VIRTIO_NET_HDR_SIZE as i32).unwrap();

        let driver_notify = SingleFdSignalQueue {
            irqfd: self.config.irqfd.clone(),
            interrupt_status: self.config.virtio.interrupt_status.clone(),
        };

        let mut ioevents = self.config.prepare_activate().map_err(Error::Virtio)?;

        let rxq = self.config.virtio.queues.remove(0);
        let txq = self.config.virtio.queues.remove(0);
        let inner = SimpleHandler::new(driver_notify, rxq, txq, tap, self.mem.clone());

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

impl<'a, M> VirtioMmioDevice for Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Send + 'static + Sync,
    <M as Deref>::Target: GuestMemory,
{
}

impl<'a, M> MutDeviceMmio for Net<'a, M>
where
    M: GuestAddressSpace<T = M> + Clone + Deref + GuestMemory + Copy + Send + 'static + Sync,
    <M as Deref>::Target: GuestMemory,
{
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
