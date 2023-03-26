use super::FileAttribute;
use crate::data_types::Align;
use crate::table::runtime::Time;
use crate::{guid, CStr16, Char16, Guid, Identify};
use core::ffi::c_void;
use core::{mem, ptr};
use ptr_meta::Pointee;

/// Errors that can occur when creating a `FileProtocolInfo`
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileInfoCreationError {
    /// The provided buffer was too small to hold the `FileInfo`. You need at
    /// least the indicated buffer size (in bytes). Please remember that using
    /// a misaligned buffer will cause a decrease of usable storage capacity.
    InsufficientStorage(usize),
}

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

impl FileInfo {
    /// Create a `FileInfo` structure
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the data structure,
    /// including a null-terminated UCS-2 `name` string.
    ///
    /// The buffer must be correctly aligned. You can query the required
    /// alignment using the `alignment()` method of the `Align` trait that this
    /// struct implements.
    #[allow(clippy::too_many_arguments)]
    pub fn new<'buf>(
        storage: &'buf mut [u8],
        file_size: u64,
        physical_size: u64,
        create_time: Time,
        last_access_time: Time,
        modification_time: Time,
        attribute: FileAttribute,
        file_name: &CStr16,
    ) -> core::result::Result<&'buf mut Self, FileInfoCreationError> {
        unsafe {
            Self::new_impl(storage, file_name, |ptr, size| {
                ptr::addr_of_mut!((*ptr).size).write(size);
                ptr::addr_of_mut!((*ptr).file_size).write(file_size);
                ptr::addr_of_mut!((*ptr).physical_size).write(physical_size);
                ptr::addr_of_mut!((*ptr).create_time).write(create_time);
                ptr::addr_of_mut!((*ptr).last_access_time).write(last_access_time);
                ptr::addr_of_mut!((*ptr).modification_time).write(modification_time);
                ptr::addr_of_mut!((*ptr).attribute).write(attribute);
            })
        }
    }

    /// File size (number of bytes stored in the file)
    #[must_use]
    pub const fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Physical space consumed by the file on the file system volume
    #[must_use]
    pub const fn physical_size(&self) -> u64 {
        self.physical_size
    }

    /// Time when the file was created
    #[must_use]
    pub const fn create_time(&self) -> &Time {
        &self.create_time
    }

    /// Time when the file was last accessed
    #[must_use]
    pub const fn last_access_time(&self) -> &Time {
        &self.last_access_time
    }

    /// Time when the file's contents were last modified
    #[must_use]
    pub const fn modification_time(&self) -> &Time {
        &self.modification_time
    }

    /// Attribute bits for the file
    #[must_use]
    pub const fn attribute(&self) -> FileAttribute {
        self.attribute
    }

    /// Name of the file
    #[must_use]
    pub fn file_name(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(self.file_name.as_ptr()) }
    }
}

impl Align for FileInfo {
    fn alignment() -> usize {
        8
    }
}

unsafe impl Identify for FileInfo {
    const GUID: Guid = guid!("09576e92-6d3f-11d2-8e39-00a0c969723b");
}

impl InfoInternal for FileInfo {
    fn name_offset() -> usize {
        80
    }
}

impl FileProtocolInfo for FileInfo {}

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

impl FileSystemInfo {
    /// Create a `FileSystemInfo` structure
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the data structure,
    /// including a null-terminated UCS-2 `name` string.
    ///
    /// The buffer must be correctly aligned. You can query the required
    /// alignment using the `alignment()` method of the `Align` trait that this
    /// struct implements.
    pub fn new<'buf>(
        storage: &'buf mut [u8],
        read_only: bool,
        volume_size: u64,
        free_space: u64,
        block_size: u32,
        volume_label: &CStr16,
    ) -> core::result::Result<&'buf mut Self, FileInfoCreationError> {
        unsafe {
            Self::new_impl(storage, volume_label, |ptr, size| {
                ptr::addr_of_mut!((*ptr).size).write(size);
                ptr::addr_of_mut!((*ptr).read_only).write(read_only);
                ptr::addr_of_mut!((*ptr).volume_size).write(volume_size);
                ptr::addr_of_mut!((*ptr).free_space).write(free_space);
                ptr::addr_of_mut!((*ptr).block_size).write(block_size);
            })
        }
    }

    /// Truth that the volume only supports read access
    #[must_use]
    pub const fn read_only(&self) -> bool {
        self.read_only
    }

    /// Number of bytes managed by the file system
    #[must_use]
    pub const fn volume_size(&self) -> u64 {
        self.volume_size
    }

    /// Number of available bytes for use by the file system
    #[must_use]
    pub const fn free_space(&self) -> u64 {
        self.free_space
    }

    /// Nominal block size by which files are typically grown
    #[must_use]
    pub const fn block_size(&self) -> u32 {
        self.block_size
    }

    /// Volume label
    #[must_use]
    pub fn volume_label(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(self.volume_label.as_ptr()) }
    }
}

impl Align for FileSystemInfo {
    fn alignment() -> usize {
        8
    }
}

unsafe impl Identify for FileSystemInfo {
    const GUID: Guid = guid!("09576e93-6d3f-11d2-8e39-00a0c969723b");
}

impl InfoInternal for FileSystemInfo {
    fn name_offset() -> usize {
        36
    }
}

impl FileProtocolInfo for FileSystemInfo {}

/// System volume label
///
/// May only be obtained on the root directory's file handle.
#[derive(Debug, Eq, PartialEq, Pointee)]
#[repr(C)]
pub struct FileSystemVolumeLabel {
    volume_label: [Char16],
}

impl FileSystemVolumeLabel {
    /// Create a `FileSystemVolumeLabel` structure
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the data structure,
    /// including a null-terminated UCS-2 `name` string.
    ///
    /// The buffer must be correctly aligned. You can query the required
    /// alignment using the `alignment()` method of the `Align` trait that this
    /// struct implements.
    pub fn new<'buf>(
        storage: &'buf mut [u8],
        volume_label: &CStr16,
    ) -> core::result::Result<&'buf mut Self, FileInfoCreationError> {
        unsafe { Self::new_impl(storage, volume_label, |_ptr, _size| {}) }
    }

    /// Volume label
    #[must_use]
    pub fn volume_label(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(self.volume_label.as_ptr()) }
    }
}

impl Align for FileSystemVolumeLabel {
    fn alignment() -> usize {
        2
    }
}

unsafe impl Identify for FileSystemVolumeLabel {
    const GUID: Guid = guid!("db47d7d3-fe81-11d3-9a35-0090273fc14d");
}

impl InfoInternal for FileSystemVolumeLabel {
    fn name_offset() -> usize {
        0
    }
}

impl FileProtocolInfo for FileSystemVolumeLabel {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::runtime::TimeParams;
    use crate::table::runtime::{Daylight, Time};
    use crate::CString16;
    use alloc::vec;

    fn validate_layout<T: InfoInternal + ?Sized>(info: &T, name: &[Char16]) {
        // Check the hardcoded struct alignment.
        assert_eq!(mem::align_of_val(info), T::alignment());
        // Check the hardcoded name slice offset.
        assert_eq!(
            unsafe { (name.as_ptr() as *const u8).offset_from(info as *const _ as *const u8) },
            T::name_offset() as isize
        );
    }

    #[test]
    fn test_file_info() {
        let mut storage = vec![0; 128];

        let file_size = 123;
        let physical_size = 456;
        let tp = TimeParams {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
            time_zone: None,
            daylight: Daylight::IN_DAYLIGHT,
        };
        let create_time = Time::new(tp).unwrap();
        let last_access_time = Time::new(TimeParams { year: 1971, ..tp }).unwrap();
        let modification_time = Time::new(TimeParams { year: 1972, ..tp }).unwrap();
        let attribute = FileAttribute::READ_ONLY;
        let name = CString16::try_from("test_name").unwrap();
        let info = FileInfo::new(
            &mut storage,
            file_size,
            physical_size,
            create_time,
            last_access_time,
            modification_time,
            attribute,
            &name,
        )
        .unwrap();

        validate_layout(info, &info.file_name);

        //   Header size: 80 bytes
        // + Name size (including trailing null): 20 bytes
        // = 100
        // Round size up to match FileInfo alignment of 8: 104
        assert_eq!(info.size, 104);
        assert_eq!(info.size, mem::size_of_val(info) as u64);

        assert_eq!(info.file_size(), file_size);
        assert_eq!(info.physical_size(), physical_size);
        assert_eq!(info.create_time(), &create_time);
        assert_eq!(info.last_access_time(), &last_access_time);
        assert_eq!(info.modification_time(), &modification_time);
        assert_eq!(info.attribute(), attribute);
        assert_eq!(info.file_name(), name);
    }

    #[test]
    fn test_file_system_info() {
        let mut storage = vec![0; 128];

        let read_only = true;
        let volume_size = 123;
        let free_space = 456;
        let block_size = 789;
        let name = CString16::try_from("test_name2").unwrap();
        let info = FileSystemInfo::new(
            &mut storage,
            read_only,
            volume_size,
            free_space,
            block_size,
            &name,
        )
        .unwrap();

        validate_layout(info, &info.volume_label);

        //   Header size: 36 bytes
        // + Name size (including trailing null): 22 bytes
        // = 58
        // Round size up to match FileSystemInfo alignment of 8: 64
        assert_eq!(info.size, 64);
        assert_eq!(info.size, mem::size_of_val(info) as u64);

        assert_eq!(info.read_only, read_only);
        assert_eq!(info.volume_size, volume_size);
        assert_eq!(info.free_space, free_space);
        assert_eq!(info.block_size, block_size);
        assert_eq!(info.volume_label(), name);
    }

    #[test]
    fn test_file_system_volume_label() {
        let mut storage = vec![0; 128];

        let name = CString16::try_from("test_name").unwrap();
        let info = FileSystemVolumeLabel::new(&mut storage, &name).unwrap();

        validate_layout(info, &info.volume_label);

        assert_eq!(info.volume_label(), name);
    }
}
