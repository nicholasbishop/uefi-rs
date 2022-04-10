use crate::uefi_stub::{install_protocol_simple, STATE};
use core::pin::Pin;
use std::ffi::c_void;
use std::{mem, ptr, slice};
use uefi::proto::media::file::{FileInfo, FileInfoCreationError};
use uefi::table::runtime::{Daylight, Time, TimeParams};
use uefi::{cstr16, CString16, Guid, Identify, Result, Status};
use uefi_raw::protocol::file_system::{
    FileAttribute, FileMode, FileProtocolRevision, FileProtocolV1 as FileProtocol,
    SimpleFileSystemProtocol,
};
use uefi_raw::{Char16, Handle};

#[repr(C)]
struct FileImpl {
    // TODO: signature
    interface: FileProtocol,
    // TODO: is there any way to make this parent pointer work with Miri?
    // fs: *mut FsImpl,
}

impl FileImpl {
    fn new() -> Self {
        Self {
            interface: FileProtocol {
                revision: FileProtocolRevision::REVISION_1,
                open,
                close,
                delete,
                read,
                write,
                get_position,
                set_position,
                get_info,
                set_info,
                flush,
            },
        }
    }
}

#[repr(C)]
pub struct FsImpl {
    interface: SimpleFileSystemProtocol,
    open_files: Vec<Pin<Box<FileImpl>>>,
}

pub type FsDb = Vec<Pin<Box<FsImpl>>>;

pub fn install_simple_file_system(handle: Handle) -> Result {
    let mut interface = ptr::null_mut();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let mut fs = Box::pin(FsImpl {
            interface: SimpleFileSystemProtocol {
                // TODO
                revision: 0,
                open_volume,
            },
            open_files: Vec::new(),
        });
        interface = ptr::addr_of_mut!(fs.interface);
        state.fs_db.push(fs);
    });

    install_protocol_simple(
        Some(handle),
        &SimpleFileSystemProtocol::GUID,
        interface.cast(),
    )?;

    Ok(())
}

unsafe extern "efiapi" fn open_volume(
    this: *mut SimpleFileSystemProtocol,
    root: *mut *mut FileProtocol,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let fs = state
            .fs_db
            .iter_mut()
            .find(|fs| this.cast_const() == ptr::addr_of!(fs.interface))
            .unwrap();

        let mut file = Box::pin(FileImpl::new());
        *root = ptr::addr_of_mut!(file.interface);
        fs.open_files.push(file);

        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn open(
    this: *mut FileProtocol,
    new_handle: *mut *mut FileProtocol,
    filename: *const Char16,
    open_mode: FileMode,
    attributes: FileAttribute,
) -> Status {
    let this_impl: *const FileImpl = this.cast();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        // TODO: not very efficient, but on the other hand UEFI won't typically
        // have many open file systems or open files.
        let fs = state
            .fs_db
            .iter_mut()
            .find(|fs| {
                fs.open_files
                    .iter()
                    .any(|f| ptr::addr_of!(**f) == this_impl)
            })
            .unwrap();

        let mut file = Box::pin(FileImpl::new());
        *new_handle = ptr::addr_of_mut!(file.interface);
        fs.open_files.push(file);

        Status::SUCCESS
    })
}

extern "efiapi" fn close(this: *mut FileProtocol) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn delete(this: *mut FileProtocol) -> Status {
    Status::SUCCESS
}

unsafe extern "efiapi" fn read(
    this: *mut FileProtocol,
    buffer_size: *mut usize,
    buffer: *mut c_void,
) -> Status {
    let mut tmp_buf = [0; 256];

    let info = FileInfo::new(
        &mut tmp_buf,
        0,
        0,
        Time::invalid(),
        Time::invalid(),
        Time::invalid(),
        FileAttribute::empty(),
        &CString16::try_from("test_dir").unwrap(),
    )
    .unwrap();

    let required_size = mem::size_of_val(info);
    let available_size = *buffer_size;
    *buffer_size = required_size;
    if available_size < required_size {
        return Status::BUFFER_TOO_SMALL;
    }

    let info_ptr: *const FileInfo = info;
    ptr::copy(info_ptr.cast::<u8>(), buffer.cast(), required_size);

    Status::SUCCESS
}

unsafe extern "efiapi" fn write(
    this: *mut FileProtocol,
    buffer_size: *mut usize,
    buffer: *const c_void,
) -> Status {
    todo!()
}

extern "efiapi" fn get_position(this: *const FileProtocol, position: *mut u64) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn set_position(this: *mut FileProtocol, position: u64) -> Status {
    Status::SUCCESS
}

unsafe extern "efiapi" fn get_info(
    this: *mut FileProtocol,
    information_type: *const Guid,
    buffer_size: *mut usize,
    buffer: *mut c_void,
) -> Status {
    let storage = slice::from_raw_parts_mut(buffer.cast::<u8>(), *buffer_size);

    match *information_type {
        FileInfo::GUID => {
            // TODO
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
            let last_access_time = create_time;
            let modification_time = create_time;
            let attribute = FileAttribute::READ_ONLY;
            let name = cstr16!("TODO");
            match FileInfo::new(
                storage,
                file_size,
                physical_size,
                create_time,
                last_access_time,
                modification_time,
                attribute,
                &name,
            ) {
                Ok(info) => {
                    *buffer_size = mem::size_of_val(info);
                    // TODO: assuming here that it was created at the start of buf,
                    // otherwise we need to memmove?
                    Status::SUCCESS
                }
                Err(FileInfoCreationError::InsufficientStorage(size)) => {
                    *buffer_size = size;
                    Status::BUFFER_TOO_SMALL
                }
            }
        }
        _ => todo!(),
    }
}

unsafe extern "efiapi" fn set_info(
    this: *mut FileProtocol,
    information_type: *const Guid,
    buffer_size: usize,
    buffer: *const c_void,
) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn flush(this: *mut FileProtocol) -> Status {
    Status::SUCCESS
}
