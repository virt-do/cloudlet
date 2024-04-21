// SPDX-License-Identifier: Apache-2.0

extern crate epoll;

use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::result;

use epoll::Events;

pub(crate) const EPOLL_EVENTS_LEN: usize = 10;

pub struct EpollContext {
    raw_fd: RawFd,
}

impl EpollContext {
    pub fn new() -> result::Result<EpollContext, io::Error> {
        let raw_fd = epoll::create(true)?;
        Ok(EpollContext { raw_fd })
    }

    pub fn add_stdin(&self) -> result::Result<(), io::Error> {
        self.add_fd(libc::STDIN_FILENO, epoll::Events::EPOLLIN)
    }

    pub fn add_fd(&self, fd: RawFd, event: Events) -> result::Result<(), io::Error> {
        epoll::ctl(
            self.raw_fd,
            epoll::ControlOptions::EPOLL_CTL_ADD,
            fd,
            epoll::Event::new(event, fd as u64),
        )?;

        Ok(())
    }
}

impl AsRawFd for EpollContext {
    fn as_raw_fd(&self) -> RawFd {
        self.raw_fd
    }
}
