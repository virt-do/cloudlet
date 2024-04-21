// SPDX-License-Identifier: Apache-2.0

use std::io;

pub(crate) mod serial;

#[derive(Debug)]
/// Devices errors.
pub enum Error {
    // Event FD creation error
    EventFdCreation(io::Error),
    // Event FD clone error
    EventFdClone(io::Error),
    // Serial raw bytes enqueueing error
    SerialEnqueue(vm_superio::serial::Error<io::Error>),
}

/// Dedicated [`Result`](https://doc.rust-lang.org/std/result/) type.
pub type Result<T> = std::result::Result<T, Error>;
