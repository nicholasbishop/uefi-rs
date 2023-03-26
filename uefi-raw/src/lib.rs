//! TODO

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]

#[cfg(feature = "alloc")]
extern crate alloc;

// allow referring to self as ::uefi for macros to work universally (from this crate and from others)
// see https://github.com/rust-lang/rust/issues/54647
extern crate self as uefi;

#[macro_use]
pub mod data_types;
#[cfg(feature = "alloc")]
pub use self::data_types::CString16;
pub use self::data_types::Identify;
pub use self::data_types::{CStr16, CStr8, Char16, Char8, Event, Guid, Handle};
pub use uefi_macros::{cstr16, cstr8, entry, guid};

mod result;
pub use self::result::{Error, Result, ResultExt, Status};

pub mod table;

pub mod proto;

mod util;
