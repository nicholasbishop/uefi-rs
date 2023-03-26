//! File system support protocols.

use super::file::FileImpl;
use crate::proto::unsafe_protocol;
use crate::Status;

/// Allows access to a FAT-12/16/32 file system.
///
/// This interface is implemented by some storage devices
/// to allow file access to the contained file systems.
///
/// # Accessing `SimpleFileSystem` protocol
///
/// Use [`BootServices::get_image_file_system`] to retrieve the `SimpleFileSystem`
/// protocol associated with a given image handle.
///
/// See the [`BootServices`] documentation for more details of how to open a protocol.
///
/// [`BootServices::get_image_file_system`]: crate::table::boot::BootServices::get_image_file_system
/// [`BootServices`]: crate::table::boot::BootServices#accessing-protocols
#[repr(C)]
#[unsafe_protocol("964e5b22-6459-11d2-8e39-00a0c969723b")]
pub struct SimpleFileSystem {
    pub revision: u64,
    pub open_volume:
        extern "efiapi" fn(this: &mut SimpleFileSystem, root: &mut *mut FileImpl) -> Status,
}
