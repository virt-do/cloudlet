// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::default::Default;

use kvm_bindings::{kvm_msr_entry, Msrs};

use crate::core::cpu::msr_index::{
    MSR_CSTAR, MSR_IA32_MISC_ENABLE, MSR_IA32_MISC_ENABLE_FAST_STRING, MSR_IA32_SYSENTER_CS,
    MSR_IA32_SYSENTER_EIP, MSR_IA32_SYSENTER_ESP, MSR_IA32_TSC, MSR_KERNEL_GS_BASE, MSR_LSTAR,
    MSR_STAR, MSR_SYSCALL_MASK,
};

// Errors associated with operations on MSRs.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Failed to initialize MSRS.
    CreateMsrs,
}

/// Specialized result type for operations on MSRs.
pub type Result<T> = std::result::Result<T, Error>;

pub fn create_boot_msr_entries() -> Result<Msrs> {
    let msr_entry_default = |msr| kvm_msr_entry {
        index: msr,
        data: 0x0,
        ..Default::default()
    };

    let raw_msrs = vec![
        msr_entry_default(MSR_IA32_SYSENTER_CS),
        msr_entry_default(MSR_IA32_SYSENTER_ESP),
        msr_entry_default(MSR_IA32_SYSENTER_EIP),
        // x86_64 specific msrs, we only run on x86_64 not x86.
        msr_entry_default(MSR_STAR),
        msr_entry_default(MSR_CSTAR),
        msr_entry_default(MSR_KERNEL_GS_BASE),
        msr_entry_default(MSR_SYSCALL_MASK),
        msr_entry_default(MSR_LSTAR),
        // end of x86_64 specific code
        msr_entry_default(MSR_IA32_TSC),
        kvm_msr_entry {
            index: MSR_IA32_MISC_ENABLE,
            data: u64::from(MSR_IA32_MISC_ENABLE_FAST_STRING),
            ..Default::default()
        },
    ];

    Msrs::from_entries(&raw_msrs).map_err(|_| Error::CreateMsrs)
}
