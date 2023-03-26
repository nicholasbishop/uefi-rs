//! `LoadedImage` protocol.

use crate::proto::device_path::FfiDevicePath;
use crate::table::boot::MemoryType;
use crate::{guid, Guid, Handle, Identify, Status};
use core::ffi::c_void;

/// The LoadedImage protocol. This can be opened on any image handle using the `HandleProtocol` boot service.
#[repr(C)]
pub struct LoadedImage {
    pub revision: u32,
    pub parent_handle: Handle,
    pub system_table: *const c_void,

    // Source location of the image
    pub device_handle: Handle,
    pub file_path: *const FfiDevicePath,
    pub reserved: *const c_void,

    // Image load options
    pub load_options_size: u32,
    pub load_options: *const u8,

    // Location where image was loaded
    pub image_base: *const c_void,
    pub image_size: u64,
    pub image_code_type: MemoryType,
    pub image_data_type: MemoryType,
    /// This is a callback that a loaded image can use to do cleanup. It is called by the
    /// `UnloadImage` boot service.
    pub unload: extern "efiapi" fn(image_handle: Handle) -> Status,
}

unsafe impl Identify for LoadedImage {
    const GUID: Guid = guid!("5b1b31a1-9562-11d2-8e3f-00a0c969723b");
}
