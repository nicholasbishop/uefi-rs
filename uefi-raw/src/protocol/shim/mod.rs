//! Shim lock protocol.

#![cfg(any(
    target_arch = "i386",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64"
))]

use crate::{guid, Guid, Status};
use core::ffi::c_void;

// The `PE_COFF_LOADER_IMAGE_CONTEXT` type.
#[repr(C)]
pub struct Context {
    pub image_address: u64,
    pub image_size: u64,
    pub entry_point: u64,
    pub size_of_headers: usize,
    pub image_type: u16,
    pub number_of_sections: u16,
    pub section_alignment: u32,
    pub first_section: *const c_void,
    pub reloc_dir: *const c_void,
    pub sec_dir: *const c_void,
    pub number_of_rva_and_sizes: u64,
    pub pe_hdr: *const c_void,
}

pub const SHA1_DIGEST_SIZE: usize = 20;
pub const SHA256_DIGEST_SIZE: usize = 32;

// These macros set the correct calling convention for the Shim protocol methods.

#[cfg(any(target_arch = "i386", target_arch = "x86_64"))]
macro_rules! shim_function {
    (fn $args:tt -> $return_type:ty) => (extern "sysv64" fn $args -> $return_type)
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
macro_rules! shim_function {
    (fn $args:tt -> $return_type:ty) => (extern "C" fn $args -> $return_type)
}

/// The Shim lock protocol.
///
/// This protocol is not part of the UEFI specification, but is
/// installed by the [Shim bootloader](https://github.com/rhboot/shim)
/// which is commonly used by Linux distributions to support UEFI
/// Secure Boot. Shim is built with an embedded certificate that is
/// used to validate another EFI application before running it. That
/// application may itself be a bootloader that needs to validate
/// another EFI application before running it, and the shim lock
/// protocol exists to support that.
#[repr(C)]
pub struct ShimLock {
    pub verify: shim_function! { fn(buffer: *const u8, size: u32) -> Status },
    pub hash: shim_function! {
        fn(
            buffer: *const u8,
            size: u32,
            context: *mut Context,
            sha256: *mut [u8; SHA256_DIGEST_SIZE],
            sha1: *mut [u8; SHA1_DIGEST_SIZE]
        ) -> Status
    },
    pub context: shim_function! { fn(buffer: *const u8, size: u32, context: *mut Context) -> Status },
}

impl ShimLock {
    pub const GUID: Guid = guid!("605dab50-e046-4300-abb6-3dd810dd8b23");
}
