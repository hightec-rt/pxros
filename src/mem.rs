//! Utilities for working with [PxOpool_t] and [PxMc_t].
//!
//! # Note
//! This is a big work in progress: PXROS supports runtime definition and
//! usage of object pools to allocate kernel objects and memory classes to
//! allocate heap.
//!
//! For now, we only expose the types but only defer to the default impl
//! when using them.
//!
//! SPDX-FileCopyrightText: Veecle GmbH, HighTec EDV-Systeme GmbH
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use core::ops::Range;

use super::bindings::*;

impl PxOpool_t {
    /// Default global object pool
    pub const GLOBAL: PxOpool_t = PxOpool_t::from_raw(PXOpoolGlobalSystemdefaultId);
    /// Default system object pool
    pub const SYSTEM: PxOpool_t = PxOpool_t::from_raw(PXOpoolSystemdefaultId);
    /// Default task object pool
    pub const TASK: PxOpool_t = PxOpool_t::from_raw(PXOpoolTaskdefaultId);
}

impl Default for PxOpool_t {
    fn default() -> Self {
        Self::TASK
    }
}

impl PxMc_t {
    /// Default system memory class
    pub const SYSTEM: PxMc_t = PxMc_t::from_raw(0x8000 + _PXMcSystemdefaultId);
    /// Default task memory class
    pub const TASK: PxMc_t = PxMc_t::from_raw(0x8000 + _PXMcTaskdefaultId);
}

impl Default for PxMc_t {
    fn default() -> Self {
        Self::TASK
    }
}

/// Type export for working with [PxProtectRegion_T]
pub type MemoryRegion = PxProtectRegion_T;

impl MemoryRegion {
    /// Creates a new [`MemoryRegion`], protecting the address range with the provided protection type.
    pub const fn new(range: Range<u32>, protection: PxProtectType_t) -> Self {
        MemoryRegion {
            lowerBound: range.start,
            upperBound: range.end,
            prot: protection as u32,
        }
    }

    /// Constructs an invalid, zeroed [`MemoryRegion`].
    ///
    /// This is typically used as the terminator or an array of [`MemoryRegion`] used in PXROS.
    pub const fn zeroed() -> Self {
        MemoryRegion {
            lowerBound: 0,
            upperBound: 0,
            prot: 0,
        }
    }
}

/// Privileges of a task for accessing peripheral blocks.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Privileges {
    /// User 0 mode
    NoDirectAccess = 0,
    /// User 1 mode
    DirectAccess = 1,
    /// Supervisor mode
    Supervisor = 2,
}

/// Utility type to work with [PxStackSpec_T] definitions
pub type StackSpec = PxStackSpec_T;

impl StackSpec {
    /// Constructs a new default stack spec with size and memory.
    pub fn new(size: u32, mem_class: PxMc_t) -> Self {
        Self {
            stk_type: PxStackSpecType_t::PXStackAlloc,
            stk_size: size / core::mem::size_of::<PxInt_t>() as u32,
            stk_src: PxStackSpec_T__bindgen_ty_1 {
                bindgen_union_field: mem_class.as_raw(),
                mc: Default::default(),
                stk: Default::default(),
            },
        }
    }
}
