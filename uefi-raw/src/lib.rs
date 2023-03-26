//! TODO

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

pub mod data_types;

pub use data_types::guid::Identify;

pub use self::data_types::Guid;
pub use uefi_macros::guid;

pub mod table;

pub mod proto;

mod status;
pub use status::Status;

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
