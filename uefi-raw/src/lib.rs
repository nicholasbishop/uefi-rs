//! TODO

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]

#[macro_use]
pub mod data_types;

pub use self::data_types::{CStr16, CStr8, Char16, Char8, Event, Guid, Handle};
pub use uefi_macros::guid;

pub mod table;

pub mod proto;

mod status;
pub use status::Status;
