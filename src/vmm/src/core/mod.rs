#![cfg(target_arch = "x86_64")]
extern crate libc;

extern crate linux_loader;
extern crate vm_memory;
extern crate vm_superio;

use linux_loader::loader;
use std::io;

use self::network::open_tap;

mod cpu;
mod devices;
mod epoll_context;
mod kernel;
mod network;
pub mod vmm;

#[derive(Debug)]

/// VMM errors.
pub enum Error {
    /// Failed to write boot parameters to guest memory.
    BootConfigure(linux_loader::configurator::Error),
    /// Error configuring the kernel command line.
    Cmdline(linux_loader::cmdline::Error),
    /// Failed to load kernel.
    KernelLoad(loader::Error),
    /// Invalid E820 configuration.
    E820Configuration,
    /// Highmem start address is past the guest memory end.
    HimemStartPastMemEnd,
    /// I/O error.
    IO(io::Error),
    /// Error issuing an ioctl to KVM.
    KvmIoctl(kvm_ioctls::Error),
    /// vCPU errors.
    Vcpu(cpu::Error),
    /// Memory error.
    Memory(vm_memory::Error),
    /// Serial creation error
    SerialCreation(io::Error),
    /// IRQ registration error
    IrqRegister(io::Error),
    /// epoll creation error
    EpollError(io::Error),
    /// STDIN read error
    StdinRead(kvm_ioctls::Error),
    /// STDIN write error
    StdinWrite(vm_superio::serial::Error<io::Error>),
    /// Terminal configuration error
    TerminalConfigure(kvm_ioctls::Error),
    // Tap open error
    OpenTap(open_tap::Error),
}

/// Dedicated [`Result`](https://doc.rust-lang.org/std/result/) type.
pub type Result<T> = std::result::Result<T, Error>;
