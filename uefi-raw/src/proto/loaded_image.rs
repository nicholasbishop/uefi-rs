//! `LoadedImage` protocol.

use crate::{
    proto::device_path::FfiDevicePath, proto::unsafe_protocol, table::boot::MemoryType, Handle,
    Status,
};
use core::ffi::c_void;

/// The LoadedImage protocol. This can be opened on any image handle using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_protocol("5b1b31a1-9562-11d2-8e3f-00a0c969723b")]
pub struct LoadedImage {
    revision: u32,
    parent_handle: Handle,
    system_table: *const c_void,

    // Source location of the image
    device_handle: Handle,
    file_path: *const FfiDevicePath,
    _reserved: *const c_void,

    // Image load options
    load_options_size: u32,
    load_options: *const u8,

    // Location where image was loaded
    image_base: *const c_void,
    image_size: u64,
    image_code_type: MemoryType,
    image_data_type: MemoryType,
    /// This is a callback that a loaded image can use to do cleanup. It is called by the
    /// `UnloadImage` boot service.
    unload: extern "efiapi" fn(image_handle: Handle) -> Status,
}
