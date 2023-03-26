//! Protocol definitions.
//!
//! Protocols are sets of related functionality identified by a unique
//! ID. They can be implemented by a UEFI driver or occasionally by a
//! UEFI application.
//!
//! See the [`BootServices`] documentation for details of how to open a
//! protocol.
//!
//! [`BootServices`]: crate::table::boot::BootServices#accessing-protocols

pub mod console;
pub mod debug;
pub mod device_path;
pub mod driver;
pub mod loaded_image;
pub mod media;
pub mod network;
pub mod pi;
pub mod rng;
pub mod security;
pub mod shim;
pub mod string;
pub mod tcg;
