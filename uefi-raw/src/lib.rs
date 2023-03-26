//! TODO

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]

// TODO: this is to make macros like `guid` work, is this OK?
extern crate self as uefi;

#[macro_use]
pub mod data_types;

pub use data_types::guid::Identify;

pub use self::data_types::{Char16, Char8, Event, Guid, Handle};
pub use uefi_macros::guid;

pub mod table;

pub mod proto;

mod status;
pub use status::Status;
