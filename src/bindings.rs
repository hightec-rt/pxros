//! Generated raw bindings.
//!
//! SPDX-FileCopyrightText: Veecle GmbH, HighTec EDV-Systeme GmbH
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use core::marker::PhantomData;

use crate::PxResult;

#[allow(unused)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::useless_transmute)]
#[allow(clippy::unnecessary_cast)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::enum_variant_names)]
#[allow(clippy::missing_safety_doc)]
pub mod generated {
    use super::*;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use generated::*;

/// Helper that implements [PxHandle] for a kernel object.
///
/// Creates a marker struct and a type alias using the marker struct with [PxHandle].
macro_rules! impl_kernel_handle {
    { $($object:ident: $marker:ident, )*} => {
        $(
            /// Marker type for a kernel object.
            #[derive(Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
            pub struct $marker;

            /// Type alias for a specialized [PxHandle].
            #[allow(non_camel_case_types)]
            pub type $object = PxHandle<$marker>;
        )*
    };
}

impl_kernel_handle! {
    PxMbx_t: PxMailbox,
    PxMsg_t: PxMessage,
    PxPe_t: PxPeriodicEvent,
    PxTask_t: PxTask,
    PxMc_t: PxMemoryClass,
    PxOpool_t: PxMemoryPool,
}

/// Typed wrapper to work with kernel handles.
///
/// This type is FFI compatible and can be safely used in FFIs.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[must_use = "Kernel handles must be used"]
pub struct PxHandle<T> {
    // Upper 16 bit are the error code.
    // Lower 16 bit are the kernel object ID.
    inner: u32,
    marker: PhantomData<T>,
}

impl<T: Copy + Clone> PxHandle<T> {
    /// Bitflag that determines if an object is valid or not.
    const INVALID_FLAG: u16 = 0x8000;

    /// Creates a new handle from a raw ID.
    pub const fn from_raw(raw: u32) -> Self {
        Self {
            inner: raw,
            marker: PhantomData,
        }
    }

    /// Returns the handle as raw u32.
    pub const fn as_raw(&self) -> u32 {
        self.inner
    }

    /// Creates a new invalid handle.
    pub const fn invalid() -> Self {
        Self {
            inner: ((PxError_t::PXERR_NOERROR as u32) << 16) | Self::INVALID_FLAG as u32,
            marker: PhantomData,
        }
    }

    /// Checks the handle has no error. Returns the error otherwise.
    pub fn checked(self) -> PxResult<Self> {
        PxResult::from(self.error()).map(|_| self)
    }

    /// Checks the handle has no error. Returns the error and self otherwise.
    pub fn checked_and_self(self) -> Result<Self, (PxError_t, Self)> {
        self.checked().map_err(|e| (e, self))
    }

    /// Returns the id of the handle.
    pub const fn id(&self) -> u16 {
        self.inner as u16
    }

    /// Returns the error of the handle.
    pub fn error(&self) -> PxError_t {
        PxError_t::from(self.inner >> 16)
    }

    /// Checks if the handle is valid.
    pub const fn is_valid(&self) -> bool {
        ((self.inner as u16) & Self::INVALID_FLAG) == 0
    }
}

#[cfg(target_arch = "tricore")]
use core::ffi;

#[cfg(not(target_arch = "tricore"))]
mod ffi {
    //! This module keeps developing the bindings on a host with the standard
    //! rust toolchain compatible with the HighTec toolchain.
    //!
    //! This is due to memory type sizes not being well defined, e.g. on `x86_64`
    //! machines the C-type `int` is 64 bits wide, so the user is forced to use
    //! the underlying rust type `i64`. Compiling the resulting implementation
    //! withthe tricore toolchain can result in compile errors because `int`
    //! in that context will translate to `i32`.

    // TODO: Hardcode the type sizes for the following types to match for the
    // HighTec compiler.
    pub use core::ffi::{
        c_char, c_int, c_longlong, c_schar, c_short, c_uchar, c_ulonglong, c_ushort, c_void,
    };

    pub use {i32 as c_long, u32 as c_uint, u32 as c_ulong};
}
