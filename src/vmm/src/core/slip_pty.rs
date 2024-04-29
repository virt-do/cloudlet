use std::{
    fs::File,
    io::Read,
    os::fd::{AsRawFd, RawFd},
    sync::Arc,
};

use nix::sys::termios;
use tracing::info;

use super::{devices::serial::LumperSerial, Error, Result};

pub struct SlipPty {
    serial: LumperSerial<Arc<File>>,
    master: Arc<File>,
}

impl SlipPty {
    pub fn new() -> Result<Self> {
        // Open a new PTY
        let (master, _, name) =
            openpty::openpty(None, None, None).map_err(|_| Error::PtyCreation)?;
        info!(?name, "Opened PTY for SLIP");

        // Disable echo in the master end
        let mut termios = termios::tcgetattr(&master).map_err(|_| Error::PtySetup)?;
        termios.local_flags.remove(termios::LocalFlags::ECHO);
        termios::tcsetattr(&master, termios::SetArg::TCSANOW, &termios)
            .map_err(|_| Error::PtySetup)?;

        // Create a new Serial device around this PTY
        let master = Arc::new(master);
        let serial = LumperSerial::new(master.clone()).map_err(Error::SerialCreation)?;

        Ok(Self {
            serial,
            master: master.clone(),
        })
    }

    pub fn serial(&self) -> &LumperSerial<Arc<File>> {
        &self.serial
    }

    pub fn serial_mut(&mut self) -> &mut LumperSerial<Arc<File>> {
        &mut self.serial
    }

    pub fn pty_master_fd(&self) -> RawFd {
        self.master.as_raw_fd()
    }

    pub fn handle_master_rx(&mut self) -> Result<()> {
        let mut out = [0u8; 1500];
        let count = self.master.read(&mut out).map_err(Error::PtyRead)?;
        self.serial
            .enqueue_raw_bytes(&out[..count])
            .map_err(Error::PtyRx)
    }
}
