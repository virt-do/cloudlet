// SPDX-License-Identifier: Apache-2.0

use std::cmp;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::ops::Deref;
use std::sync::Arc;

use super::{Error, Result};

use vm_superio::serial::SerialEvents;
use vm_superio::{Serial, Trigger};
use vmm_sys_util::eventfd::EventFd;

pub const SERIAL_PORT_BASE: u16 = 0x3f8;
pub const SERIAL_PORT_LAST_REGISTER: u16 = SERIAL_PORT_BASE + 0x8;

pub const SERIAL2_PORT_BASE: u16 = 0x2f8;
pub const SERIAL2_PORT_LAST_REGISTER: u16 = SERIAL2_PORT_BASE + 0x8;

pub struct EventFdTrigger(EventFd);

impl Trigger for EventFdTrigger {
    type E = io::Error;

    fn trigger(&self) -> io::Result<()> {
        self.write(1)
    }
}

impl Deref for EventFdTrigger {
    type Target = EventFd;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EventFdTrigger {
    pub fn new(flag: i32) -> io::Result<Self> {
        Ok(EventFdTrigger(EventFd::new(flag)?))
    }
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(EventFdTrigger((**self).try_clone()?))
    }
}

pub(crate) struct LumperSerial<W: Write> {
    // evenfd allows for the device to send interrupts to the guest.
    eventfd: EventFdTrigger,

    // serial is the actual serial device.
    pub serial: Serial<EventFdTrigger, LumperSerialEvents, W>,

    in_buffer: VecDeque<u8>,
    in_buffer_empty_eventfd: Arc<EventFd>,
}

impl<W: Write> LumperSerial<W> {
    pub fn new(out: W) -> Result<Self> {
        let eventfd = EventFdTrigger::new(libc::EFD_NONBLOCK).unwrap();
        let in_buffer_empty_eventfd =
            Arc::new(EventFd::new(libc::EFD_NONBLOCK).map_err(Error::EventFdCreation)?);
        let events: LumperSerialEvents = LumperSerialEvents::new(in_buffer_empty_eventfd.clone());

        Ok(LumperSerial {
            eventfd: eventfd.try_clone().map_err(Error::EventFdClone)?,
            serial: Serial::with_events(
                eventfd.try_clone().map_err(Error::EventFdClone)?,
                events,
                out,
            ),
            in_buffer: VecDeque::new(),
            in_buffer_empty_eventfd,
        })
    }

    pub fn eventfd(&self) -> Result<EventFd> {
        Ok(self.eventfd.try_clone().map_err(Error::EventFdClone)?.0)
    }

    pub fn enqueue_raw_bytes(&mut self, input: &[u8]) -> Result<()> {
        self.in_buffer.extend(input);
        self.flush_in_buffer()
    }

    pub fn flush_in_buffer(&mut self) -> Result<()> {
        let fifo_capacity = self.serial.fifo_capacity();

        if fifo_capacity > 0 {
            let drained = self
                .in_buffer
                .drain(..cmp::min(self.in_buffer.len(), fifo_capacity))
                .collect::<Vec<u8>>();

            self.serial
                .enqueue_raw_bytes(&drained)
                .map_err(Error::SerialEnqueue)?;
        }

        Ok(())
    }

    pub fn in_buffer_empty_eventfd(&self) -> &EventFd {
        &self.in_buffer_empty_eventfd
    }
}

pub(crate) struct LumperSerialEvents {
    in_buffer_empty_eventfd: Arc<EventFd>,
}

impl LumperSerialEvents {
    pub fn new(in_buffer_empty_eventfd: Arc<EventFd>) -> Self {
        Self {
            in_buffer_empty_eventfd,
        }
    }
}

impl SerialEvents for LumperSerialEvents {
    fn buffer_read(&self) {}

    fn out_byte(&self) {}

    fn tx_lost_byte(&self) {}

    fn in_buffer_empty(&self) {
        self.in_buffer_empty_eventfd.write(1).unwrap();
    }
}
