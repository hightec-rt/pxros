//! PXROS Rust wrappers around the bindings.
//!
//! This crate "wraps" the raw bindings in Rust data structures and provides a more
//! rusty interface. It minimizes logical changes.
//! 
//! SPDX-FileCopyrightText: Veecle GmbH, HighTec EDV-Systeme GmbH
//! 
//! SPDX-License-Identifier: Apache-2.0
//! 
#![cfg_attr(not(test), no_std)]

use bindings::PxError_t;
use defmt::Formatter;

use crate::bindings::PxEvents_t;

pub mod bindings;
pub mod mem;

/// Specialized result for a [PxError_t] with utility
/// methods from/into
pub type PxResult<T> = core::result::Result<T, PxError_t>;

impl From<PxError_t> for PxResult<()> {
    fn from(value: PxError_t) -> Self {
        if value == PxError_t::PXERR_NOERROR {
            Ok(())
        } else {
            Err(value)
        }
    }
}

impl From<u64> for PxError_t {
    fn from(value: u64) -> Self {
        Self::from(value as u32)
    }
}

impl From<u32> for PxError_t {
    fn from(value: u32) -> Self {
        if value < PxError_t::PXERR_LAST_ERRNO as u32 {
            // # Safety
            // Value is less than last error number and as such must be defined.
            unsafe { core::mem::transmute(value) }
        } else {
            panic!("Value not within defined PxError_t range");
        }
    }
}

impl defmt::Format for PxEvents_t {
    fn format(&self, fmt: Formatter) {
        defmt::write!(fmt, "{:032b}", self.0)
    }
}
