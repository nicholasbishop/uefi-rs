//! Shim lock protocol.

#![cfg(any(
    target_arch = "i386",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64"
))]

use crate::proto::unsafe_protocol;
use crate::Status;
use core::ffi::c_void;

// The `PE_COFF_LOADER_IMAGE_CONTEXT` type. None of our methods need to inspect
// the fields of this struct, we just need to make sure it is the right size.
#[repr(C)]
pub struct Context {
    pub _image_address: u64,
    pub _image_size: u64,
    pub _entry_point: u64,
    pub _size_of_headers: usize,
    pub _image_type: u16,
    pub _number_of_sections: u16,
    pub _section_alignment: u32,
    pub _first_section: *const c_void,
    pub _reloc_dir: *const c_void,
    pub _sec_dir: *const c_void,
    pub _number_of_rva_and_sizes: u64,
    pub _pe_hdr: *const c_void,
}

pub const SHA1_DIGEST_SIZE: usize = 20;
pub const SHA256_DIGEST_SIZE: usize = 32;

/// Authenticode hashes of some UEFI application.
#[derive(Debug)]
pub struct Hashes {
    /// SHA256 Authenticode Digest
    pub sha256: [u8; SHA256_DIGEST_SIZE],
    /// SHA1 Authenticode Digest
    pub sha1: [u8; SHA1_DIGEST_SIZE],
}

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
#[unsafe_protocol("605dab50-e046-4300-abb6-3dd810dd8b23")]
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
