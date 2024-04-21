use vm_memory::{GuestAddress, GuestUsize};

use crate::core::devices::virtio::register_mmio_device;

pub struct Net {
    vmmio_parameter: String,
}

impl Net {
    pub fn new(
        size: GuestUsize,
        baseaddr: GuestAddress,
        irq: u32,
    ) -> Result<Self, linux_loader::cmdline::Error> {
        let vmmio_parameter = register_mmio_device(size, baseaddr, irq, None).unwrap();

        Ok(Self { vmmio_parameter })
    }

    pub fn get_vmmio_parameter(&self) -> String {
        self.vmmio_parameter.clone()
    }
}
