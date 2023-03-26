use super::FileAttribute;
use crate::table::runtime::Time;
use crate::{guid, Char16, Guid, Identify};
use ptr_meta::Pointee;

/// Generic file information
///
/// The following rules apply when using this struct with `set_info()`:
///
/// - On directories, the file size is determined by the contents of the
///   directory and cannot be changed by setting `file_size`. This member is
///   ignored by `set_info()`.
/// - The `physical_size` is determined by the `file_size` and cannot be
///   changed. This member is ignored by `set_info()`.
/// - The `FileAttribute::DIRECTORY` bit cannot be changed. It must match the
///   file’s actual type.
/// - A value of zero in create_time, last_access, or modification_time causes
///   the fields to be ignored (and not updated).
/// - It is forbidden to change the name of a file to the name of another
///   existing file in the same directory.
/// - If a file is read-only, the only allowed change is to remove the read-only
///   attribute. Other changes must be carried out in a separate transaction.
#[derive(Debug, Eq, PartialEq, Pointee)]
#[repr(C)]
pub struct FileInfo {
    size: u64,
    file_size: u64,
    physical_size: u64,
    create_time: Time,
    last_access_time: Time,
    modification_time: Time,
    attribute: FileAttribute,
    file_name: [Char16],
}

unsafe impl Identify for FileInfo {
    const GUID: Guid = guid!("09576e92-6d3f-11d2-8e39-00a0c969723b");
}

/// System volume information
///
/// May only be obtained on the root directory's file handle.
///
/// Please note that only the system volume's volume label may be set using
/// this information structure. Consider using `FileSystemVolumeLabel` instead.
#[derive(Debug, Eq, PartialEq, Pointee)]
#[repr(C)]
pub struct FileSystemInfo {
    size: u64,
    read_only: bool,
    volume_size: u64,
    free_space: u64,
    block_size: u32,
    volume_label: [Char16],
}

unsafe impl Identify for FileSystemInfo {
    const GUID: Guid = guid!("09576e93-6d3f-11d2-8e39-00a0c969723b");
}

/// System volume label
///
/// May only be obtained on the root directory's file handle.
#[derive(Debug, Eq, PartialEq, Pointee)]
#[repr(C)]
pub struct FileSystemVolumeLabel {
    volume_label: [Char16],
}

unsafe impl Identify for FileSystemVolumeLabel {
    const GUID: Guid = guid!("db47d7d3-fe81-11d3-9a35-0090273fc14d");
}
