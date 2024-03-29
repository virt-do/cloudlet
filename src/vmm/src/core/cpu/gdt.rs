// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

// For GDT details see arch/x86/include/asm/segment.h

use kvm_bindings::kvm_segment;
use std::mem;
use vm_memory::{Bytes, GuestAddress, GuestMemory, GuestMemoryMmap};

pub const BOOT_GDT_OFFSET: u64 = 0x500;
pub const BOOT_IDT_OFFSET: u64 = 0x520;
pub const BOOT_GDT_MAX: usize = 4;

pub fn gdt_entry(flags: u16, base: u32, limit: u32) -> u64 {
    ((u64::from(base) & 0xff00_0000u64) << (56 - 24))
        | ((u64::from(flags) & 0x0000_f0ffu64) << 40)
        | ((u64::from(limit) & 0x000f_0000u64) << (48 - 16))
        | ((u64::from(base) & 0x00ff_ffffu64) << 16)
        | (u64::from(limit) & 0x0000_ffffu64)
}

pub fn get_base(entry: u64) -> u64 {
    (((entry) & 0xFF00_0000_0000_0000) >> 32)
        | (((entry) & 0x0000_00FF_0000_0000) >> 16)
        | (((entry) & 0x0000_0000_FFFF_0000) >> 16)
}

pub fn get_limit(entry: u64) -> u32 {
    ((((entry) & 0x000F_0000_0000_0000) >> 32) | ((entry) & 0x0000_0000_0000_FFFF)) as u32
}

pub fn get_g(entry: u64) -> u8 {
    ((entry & 0x0080_0000_0000_0000) >> 55) as u8
}

pub fn get_db(entry: u64) -> u8 {
    ((entry & 0x0040_0000_0000_0000) >> 54) as u8
}

pub fn get_l(entry: u64) -> u8 {
    ((entry & 0x0020_0000_0000_0000) >> 53) as u8
}

pub fn get_avl(entry: u64) -> u8 {
    ((entry & 0x0010_0000_0000_0000) >> 52) as u8
}

pub fn get_p(entry: u64) -> u8 {
    ((entry & 0x0000_8000_0000_0000) >> 47) as u8
}

pub fn get_dpl(entry: u64) -> u8 {
    ((entry & 0x0000_6000_0000_0000) >> 45) as u8
}

pub fn get_s(entry: u64) -> u8 {
    ((entry & 0x0000_1000_0000_0000) >> 44) as u8
}

pub fn get_type(entry: u64) -> u8 {
    ((entry & 0x0000_0F00_0000_0000) >> 40) as u8
}

/// Automatically build the kvm struct for SET_SREGS from the kernel bit fields.
///
/// # Arguments
///
/// * `entry` - The gdt entry.
/// * `table_index` - Index of the entry in the gdt table.
pub fn kvm_segment_from_gdt(entry: u64, table_index: u8) -> kvm_segment {
    kvm_segment {
        base: get_base(entry),
        limit: get_limit(entry),
        selector: u16::from(table_index * 8),
        type_: get_type(entry),
        present: get_p(entry),
        dpl: get_dpl(entry),
        db: get_db(entry),
        s: get_s(entry),
        l: get_l(entry),
        g: get_g(entry),
        avl: get_avl(entry),
        padding: 0,
        unusable: match get_p(entry) {
            0 => 1,
            _ => 0,
        },
    }
}

pub fn write_gdt_table(
    table: &[u64],
    guest_mem: &GuestMemoryMmap,
) -> std::result::Result<(), vm_memory::GuestMemoryError> {
    let boot_gdt_addr = GuestAddress(BOOT_GDT_OFFSET);
    for (index, entry) in table.iter().enumerate() {
        let addr = guest_mem
            .checked_offset(boot_gdt_addr, index * mem::size_of::<u64>())
            .ok_or(vm_memory::GuestMemoryError::InvalidGuestAddress(
                boot_gdt_addr,
            ))?;
        guest_mem.write_obj(*entry, addr)?;
    }
    Ok(())
}

pub fn write_idt_value(
    val: u64,
    guest_mem: &GuestMemoryMmap,
) -> std::result::Result<(), vm_memory::GuestMemoryError> {
    let boot_idt_addr = GuestAddress(BOOT_IDT_OFFSET);
    guest_mem.write_obj(val, boot_idt_addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn field_parse() {
        let gdt = gdt_entry(0xA09B, 0x10_0000, 0xfffff);
        let seg = kvm_segment_from_gdt(gdt, 0);
        // 0xA09B
        // 'A'
        assert_eq!(0x1, seg.g);
        assert_eq!(0x0, seg.db);
        assert_eq!(0x1, seg.l);
        assert_eq!(0x0, seg.avl);
        // '9'
        assert_eq!(0x1, seg.present);
        assert_eq!(0x0, seg.dpl);
        assert_eq!(0x1, seg.s);
        // 'B'
        assert_eq!(0xB, seg.type_);
        // base and limit
        assert_eq!(0x10_0000, seg.base);
        assert_eq!(0xfffff, seg.limit);
        assert_eq!(0x0, seg.unusable);
    }
}
