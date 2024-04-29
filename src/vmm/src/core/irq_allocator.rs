// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidValue,
    MaxIrq,
    IRQOverflowed,
}

pub type Result<T> = std::result::Result<T, Error>;

/// An irq allocator which gives next available irq.
/// It is mainly used for non-legacy devices.
// There are a few reserved irq's on x86_64. We just skip all the inital
// reserved irq to make the implementaion simple. This could be later extended
// to cater more complex scenario.
#[derive(Debug)]
pub struct IrqAllocator {
    // Tracks the last allocated irq
    last_used_irq: u32,
    last_irq: u32,
}

impl IrqAllocator {
    pub fn new(last_used_irq: u32, last_irq: u32) -> Result<Self> {
        if last_used_irq >= last_irq {
            return Err(Error::InvalidValue);
        }
        Ok(IrqAllocator {
            last_used_irq,
            last_irq,
        })
    }

    pub fn next_irq(&mut self) -> Result<u32> {
        self.last_used_irq
            .checked_add(1)
            .ok_or(Error::IRQOverflowed)
            .and_then(|irq| {
                if irq > self.last_irq {
                    Err(Error::MaxIrq)
                } else {
                    self.last_used_irq = irq;
                    Ok(irq)
                }
            })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Error::MaxIrq => "last_irq IRQ limit reached",
            Error::IRQOverflowed => "IRQ overflowed",
            Error::InvalidValue => {
                "Check the value of last_used and last_irq. las_used should be less than last_irq"
            }
        };
        write!(f, "{}", err) // user-facing output
    }
}
