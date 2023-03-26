//! This module provides the `FileHandle` structure as well as the more specific `RegularFile` and
//! `Directory` structures. This module also provides the `File` trait for opening, querying,
//! creating, reading, and writing files.
//!
//! Usually a file system implementation will return a "root" directory, representing
//! `/` on that volume. With that directory, it is possible to enumerate and open
//! all the other files on that volume.

mod info;

use crate::{Char16, Guid, Status};
use bitflags::bitflags;
use core::ffi::c_void;
use core::fmt::Debug;

pub use self::info::{FileInfo, FileSystemInfo, FileSystemVolumeLabel};

/// An opaque handle to some contiguous block of data on a volume.
///
/// A `FileHandle` is just a wrapper around a UEFI file handle. Under the hood, it can either be a
/// `RegularFile` or a `Directory`; use the `into_type()` or the unsafe
/// `{RegularFile, Directory}::new()` methods to perform the conversion.
///
/// Dropping this structure will result in the file handle being closed.
#[repr(transparent)]
#[derive(Debug)]
pub struct FileHandle(pub *mut FileImpl);

/// The function pointer table for the File protocol.
#[repr(C)]
pub struct FileImpl {
    pub revision: u64,
    pub open: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        new_handle: &mut *mut FileImpl,
        filename: *const Char16,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Status,
    pub close: extern "efiapi" fn(this: &mut FileImpl) -> Status,
    pub delete: extern "efiapi" fn(this: &mut FileImpl) -> Status,
    /// # Read from Regular Files
    /// If `self` is not a directory, the function reads the requested number of bytes from the file
    /// at the file’s current position and returns them in `buffer`. If the read goes beyond the end
    /// of the file, the read length is truncated to the end of the file. The file’s current
    /// position is increased by the number of bytes returned.
    ///
    /// # Read from Directory
    /// If `self` is a directory, the function reads the directory entry at the file’s current
    /// position and returns the entry in `buffer`. If the `buffer` is not large enough to hold the
    /// current directory entry, then `EFI_BUFFER_TOO_SMALL` is returned and the current file
    /// position is not updated. `buffer_size` is set to be the size of the buffer needed to read
    /// the entry. On success, the current position is updated to the next directory entry. If there
    /// are no more directory entries, the read returns a zero-length buffer.
    pub read: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        buffer_size: &mut usize,
        buffer: *mut u8,
    ) -> Status,
    pub write: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        buffer_size: &mut usize,
        buffer: *const u8,
    ) -> Status,
    pub get_position: extern "efiapi" fn(this: &mut FileImpl, position: &mut u64) -> Status,
    pub set_position: extern "efiapi" fn(this: &mut FileImpl, position: u64) -> Status,
    pub get_info: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        information_type: &Guid,
        buffer_size: &mut usize,
        buffer: *mut u8,
    ) -> Status,
    pub set_info: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        information_type: &Guid,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
    pub flush: extern "efiapi" fn(this: &mut FileImpl) -> Status,
}

/// Usage flags describing what is possible to do with the file.
///
/// SAFETY: Using a repr(C) enum is safe here because this type is only sent to
///         the UEFI implementation, and never received from it.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u64)]
pub enum FileMode {
    /// The file can be read from
    Read = 1,

    /// The file can be read from and written to
    ReadWrite = 2 | 1,

    /// The file can be read, written, and will be created if it does not exist
    CreateReadWrite = (1 << 63) | 2 | 1,
}

bitflags! {
    /// Attributes describing the properties of a file on the file system.
    #[repr(transparent)]
    pub struct FileAttribute: u64 {
        /// File can only be opened in [`FileMode::READ`] mode.
        const READ_ONLY = 1;
        /// Hidden file, not normally visible to the user.
        const HIDDEN = 1 << 1;
        /// System file, indicates this file is an internal operating system file.
        const SYSTEM = 1 << 2;
        /// This file is a directory.
        const DIRECTORY = 1 << 4;
        /// This file is compressed.
        const ARCHIVE = 1 << 5;
        /// Mask combining all the valid attributes.
        const VALID_ATTR = 0x37;
    }
}
