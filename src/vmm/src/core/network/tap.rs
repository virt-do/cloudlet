// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

use super::{create_inet_socket, create_sockaddr, create_unix_socket, Error as NetUtilError};
use crate::core::network::mac::MacAddr;
use crate::core::network::mac::MAC_ADDR_LEN;
use crate::core::network::net_gen;
use std::fs::File;
use std::io::{Error as IoError, Read, Result as IoResult, Write};
use std::net;
use std::os::raw::*;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use vmm_sys_util::ioctl::{ioctl_with_mut_ref, ioctl_with_ref};

#[derive(Debug)]
pub enum Error {
    OpenTun(IoError),
    ConfigureTap(IoError),
    GetFeatures(IoError),
    MultiQueueKernelSupport,
    Ioctl(c_ulong, IoError),
    NetUtil(NetUtilError),
    InvalidIfname,
    MacParsing(IoError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// Handle for a network tap interface.
///
/// For now, this simply wraps the file descriptor for the tap device so methods
/// can run ioctls on the interface. The tap interface fd will be closed when
/// Tap goes out of scope, and the kernel will clean up the interface
/// automatically.
#[derive(Debug)]
pub struct Tap {
    tap_file: File,
    if_name: Vec<u8>,
}

impl PartialEq for Tap {
    fn eq(&self, other: &Tap) -> bool {
        self.if_name == other.if_name
    }
}

impl std::clone::Clone for Tap {
    fn clone(&self) -> Self {
        Tap {
            tap_file: self.tap_file.try_clone().unwrap(),
            if_name: self.if_name.clone(),
        }
    }
}

// Returns a byte vector representing the contents of a null terminated C string which
// contains if_name.
fn build_terminated_if_name(if_name: &str) -> Result<Vec<u8>> {
    // Convert the string slice to bytes, and shadow the variable,
    // since we no longer need the &str version.
    let if_name = if_name.as_bytes();

    // TODO: the 16usize limit of the if_name member from struct Tap is pretty arbitrary.
    // We leave it as is for now, but this should be refactored at some point.
    if if_name.len() > 15 {
        return Err(Error::InvalidIfname);
    }

    let mut terminated_if_name = vec![b'\0'; if_name.len() + 1];
    terminated_if_name[..if_name.len()].copy_from_slice(if_name);

    Ok(terminated_if_name)
}

impl Tap {
    unsafe fn ioctl_with_ref<F: AsRawFd, T>(fd: &F, req: c_ulong, arg: &T) -> Result<()> {
        let ret = ioctl_with_ref(fd, req, arg);
        if ret < 0 {
            return Err(Error::Ioctl(req, IoError::last_os_error()));
        }

        Ok(())
    }

    pub fn open_named(if_name: &str, num_queue_pairs: usize, flags: Option<i32>) -> Result<Tap> {
        let terminated_if_name = build_terminated_if_name(if_name)?;

        // SAFETY: FFI call
        let fd = unsafe {
            // Open calls are safe because we give a constant null-terminated
            // string and verify the result.
            libc::open(
                b"/dev/net/tun\0".as_ptr() as *const c_char,
                flags.unwrap_or(libc::O_RDWR | libc::O_NONBLOCK | libc::O_CLOEXEC),
            )
        };
        if fd < 0 {
            return Err(Error::OpenTun(IoError::last_os_error()));
        }

        // SAFETY: We just checked that the fd is valid.
        let tuntap = unsafe { File::from_raw_fd(fd) };

        // Let's validate some features before going any further.
        // ioctl is safe since we call it with a valid tap fd and check the return
        // value.
        let mut features = 0;
        // SAFETY: IOCTL with correct arguments
        let ret = unsafe { ioctl_with_mut_ref(&tuntap, net_gen::TUNGETFEATURES(), &mut features) };
        if ret < 0 {
            return Err(Error::GetFeatures(IoError::last_os_error()));
        }

        // Check if the user parameters match the kernel support for MQ
        if (features & net_gen::IFF_MULTI_QUEUE == 0) && num_queue_pairs > 1 {
            return Err(Error::MultiQueueKernelSupport);
        }

        // This is pretty messy because of the unions used by ifreq. Since we
        // don't call as_mut on the same union field more than once, this block
        // is safe.
        let mut ifreq: net_gen::ifreq = Default::default();
        // SAFETY: see the comment above.
        unsafe {
            let ifrn_name = ifreq.ifr_ifrn.ifrn_name.as_mut();
            let name_slice = &mut ifrn_name[..terminated_if_name.len()];
            name_slice.copy_from_slice(terminated_if_name.as_slice());
            ifreq.ifr_ifru.ifru_flags =
                (net_gen::IFF_TAP | net_gen::IFF_NO_PI | net_gen::IFF_VNET_HDR) as c_short;
            if num_queue_pairs > 1 {
                ifreq.ifr_ifru.ifru_flags |= net_gen::IFF_MULTI_QUEUE as c_short;
            }
        }

        // SAFETY: ioctl is safe since we call it with a valid tap fd and check the return
        // value.
        let ret = unsafe { ioctl_with_mut_ref(&tuntap, net_gen::TUNSETIFF(), &mut ifreq) };
        if ret < 0 {
            return Err(Error::ConfigureTap(IoError::last_os_error()));
        }

        // SAFETY: only the name is accessed, and it's cloned out.
        let mut if_name = unsafe { ifreq.ifr_ifrn.ifrn_name }.to_vec();
        if_name.truncate(terminated_if_name.len() - 1);
        Ok(Tap {
            tap_file: tuntap,
            if_name,
        })
    }

    /// Create a new tap interface.
    pub fn new(num_queue_pairs: usize) -> Result<Tap> {
        Self::open_named("taplet%d", num_queue_pairs, None)
    }

    /// Set the host-side IP address for the tap interface.
    pub fn set_ip_addr(&self, ip_addr: net::Ipv4Addr) -> Result<()> {
        let sock = create_inet_socket().map_err(Error::NetUtil)?;
        let addr = create_sockaddr(ip_addr);

        let mut ifreq = self.get_ifreq();

        ifreq.ifr_ifru.ifru_addr = addr;

        // SAFETY: ioctl is safe. Called with a valid sock fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCSIFADDR as c_ulong, &ifreq) }
    }

    /// Set mac addr for tap interface.
    pub fn set_mac_addr(&self, addr: MacAddr) -> Result<()> {
        // Checking if the mac address already matches the desired one
        // is useful to avoid making the "set ioctl" in the case where
        // the VMM is running without the privilege to do that.
        // In practice this comes from a reboot after the configuration
        // has been update with the kernel generated address.
        if self.get_mac_addr()? == addr {
            return Ok(());
        }

        let sock = create_unix_socket().map_err(Error::NetUtil)?;

        let mut ifreq = self.get_ifreq();

        // SAFETY: ioctl is safe. Called with a valid sock fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCGIFHWADDR as c_ulong, &ifreq)? };

        // SAFETY: We only access one field of the ifru union
        unsafe {
            let ifru_hwaddr = &mut ifreq.ifr_ifru.ifru_hwaddr;
            for (i, v) in addr.get_bytes().iter().enumerate() {
                ifru_hwaddr.sa_data[i] = *v as c_uchar;
            }
        }

        // SAFETY: ioctl is safe. Called with a valid sock fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCSIFHWADDR as c_ulong, &ifreq) }
    }

    /// Get mac addr for tap interface.
    pub fn get_mac_addr(&self) -> Result<MacAddr> {
        let sock = create_unix_socket().map_err(Error::NetUtil)?;

        let ifreq = self.get_ifreq();

        // SAFETY: ioctl is safe. Called with a valid sock fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCGIFHWADDR as c_ulong, &ifreq)? };

        // SAFETY: We only access one field of the ifru union
        let addr = unsafe {
            MacAddr::from_bytes(&ifreq.ifr_ifru.ifru_hwaddr.sa_data[0..MAC_ADDR_LEN])
                .map_err(Error::MacParsing)?
        };
        Ok(addr)
    }

    /// Set the netmask for the subnet that the tap interface will exist on.
    pub fn set_netmask(&self, netmask: net::Ipv4Addr) -> Result<()> {
        let sock = create_inet_socket().map_err(Error::NetUtil)?;
        let addr = create_sockaddr(netmask);

        let mut ifreq = self.get_ifreq();

        ifreq.ifr_ifru.ifru_addr = addr;

        // SAFETY: ioctl is safe. Called with a valid sock fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCSIFNETMASK as c_ulong, &ifreq) }
    }

    pub fn set_mtu(&self, mtu: i32) -> Result<()> {
        let sock = create_unix_socket().map_err(Error::NetUtil)?;

        let mut ifreq = self.get_ifreq();
        ifreq.ifr_ifru.ifru_mtu = mtu;

        // SAFETY: ioctl is safe. Called with a valid sock fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCSIFMTU as c_ulong, &ifreq) }
    }

    // TODO: Use that when the virtio-net will be implemented, on device activation
    // https://github.com/rust-vmm/vmm-reference/blob/89e4c8ba56b553eeefe53ab318a67b870a8b8e41/src/devices/src/virtio/net/device.rs#L104
    //
    // /// Set the offload flags for the tap interface.
    // pub fn set_offload(&self, flags: c_uint) -> Result<()> {
    //     // SAFETY: ioctl is safe. Called with a valid tap fd, and we check the return.
    //     unsafe { Self::ioctl_with_val(&self.tap_file, net_gen::TUNSETOFFLOAD(), flags as c_ulong) }
    // }

    /// Enable the tap interface.
    pub fn enable(&self) -> Result<()> {
        let sock = create_unix_socket().map_err(Error::NetUtil)?;

        let mut ifreq = self.get_ifreq();

        // SAFETY: IOCTL with correct arguments
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCGIFFLAGS as c_ulong, &ifreq)? };

        // If TAP device is already up don't try and enable it
        // SAFETY: access a union field
        let ifru_flags = unsafe { ifreq.ifr_ifru.ifru_flags };
        if ifru_flags & net_gen::net_device_flags_IFF_UP as i16
            == net_gen::net_device_flags_IFF_UP as i16
        {
            return Ok(());
        }

        ifreq.ifr_ifru.ifru_flags = net_gen::net_device_flags_IFF_UP as i16;

        // SAFETY: ioctl is safe. Called with a valid sock fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&sock, net_gen::sockios::SIOCSIFFLAGS as c_ulong, &ifreq) }
    }

    /// Set the size of the vnet hdr.
    pub fn set_vnet_hdr_size(&self, size: c_int) -> Result<()> {
        // SAFETY: ioctl is safe. Called with a valid tap fd, and we check the return.
        unsafe { Self::ioctl_with_ref(&self.tap_file, net_gen::TUNSETVNETHDRSZ(), &size) }
    }

    fn get_ifreq(&self) -> net_gen::ifreq {
        let mut ifreq: net_gen::ifreq = Default::default();

        // This sets the name of the interface, which is the only entry
        // in a single-field union.
        // SAFETY: access union fields and we're sure the copy is okay.
        unsafe {
            let ifrn_name = ifreq.ifr_ifrn.ifrn_name.as_mut();
            let name_slice = &mut ifrn_name[..self.if_name.len()];
            name_slice.copy_from_slice(&self.if_name);
        }

        ifreq
    }
}

impl Read for Tap {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.tap_file.read(buf)
    }
}

impl Write for Tap {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.tap_file.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

impl AsRawFd for Tap {
    fn as_raw_fd(&self) -> RawFd {
        self.tap_file.as_raw_fd()
    }
}
