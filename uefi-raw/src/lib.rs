//! Raw interface for working with UEFI.
//!
//! This crate is intended for implementing UEFI services. It is also used for
//! implementing the [`uefi`] crate, which provides a safe wrapper around UEFI.
//!
//! For creating UEFI applications and drivers, consider using the [`uefi`]
//! crate instead of `uefi-raw`.
//!
//! [`uefi`]: https://crates.io/crates/uefi

#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// Enable some additional warnings and lints.
#![deny(clippy::ptr_as_ptr, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]

// Allow the uefi-raw crate to internally access itself under the "uefi" name so
// that macros like `guid!` work.
extern crate self as uefi;

#[macro_use]
mod enums;

pub mod protocol;
pub mod result;
pub mod table;

mod guid;
mod status;

pub use guid::Guid;
pub use status::Status;
pub use uefi_macros::guid;
