//! Raw interface for working with UEFI.
//!
//! This crate is intended for implementing UEFI services. It is also used for
//! implementing the [`uefi`] crate, which provides a safe wrapper around UEFI.
//!
//! For creating UEFI applications and drivers, consider using the [`uefi`]
//! crate instead of `uefi-raw`.
//!
//! [`uefi`]: https://crates.io/crates/uefi

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]

use core::ffi::c_void;

// TODO: this is to make macros like `guid` work, is this OK?
extern crate self as uefi;

#[macro_use]
mod enums;

pub mod proto;
pub mod table;

mod guid;
mod result;

pub use guid::{Guid, Identify};
pub use result::{Error, Result, ResultExt, Status};
pub use uefi_macros::guid;

/// A Latin-1 character
pub type Char8 = u8;

/// An UCS-2 code point
pub type Char16 = u16;

/// Opaque handle to a UEFI entity.
pub type Handle = *const c_void;

/// Opaque handle to a UEFI event.
pub type Event = *const c_void;

/// Physical memory address. This is always a 64-bit value, regardless
/// of target platform.
pub type PhysicalAddress = u64;

/// Virtual memory address. This is always a 64-bit value, regardless
/// of target platform.
pub type VirtualAddress = u64;
