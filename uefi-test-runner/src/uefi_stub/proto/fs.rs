use crate::uefi_stub::{install_owned_protocol, SharedAnyBox, STATE};
use std::collections::HashMap;
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
    fs: *mut FsImpl,
}

impl FileImpl {
    fn new(fs: *mut FsImpl) -> Self {
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
            fs,
        }
    }
}

pub struct FsImpl {
    open_files: Vec<Box<FileImpl>>,
}

pub type FsDb = HashMap<*const SimpleFileSystemProtocol, Box<FsImpl>>;

pub fn install_simple_file_system(handle: Handle) -> Result {
    let mut interface = SharedAnyBox::new(SimpleFileSystemProtocol {
        revision: 0,
        open_volume,
    });
    let sfs = interface.as_mut_ptr();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        // TODO: move fs_db into protocol owned data?
        state.fs_db.insert(
            sfs.cast(),
            Box::new(FsImpl {
                open_files: Vec::new(),
            }),
        );
    });

    install_owned_protocol(
        Some(handle),
        SimpleFileSystemProtocol::GUID,
        sfs.cast(),
        interface,
        None,
    )?;

    Ok(())
}

unsafe extern "efiapi" fn open_volume(
    this: *mut SimpleFileSystemProtocol,
    root: *mut *mut FileProtocol,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let this: *const SimpleFileSystemProtocol = this;
        let fs_impl = state.fs_db.get_mut(&this).unwrap();
        let mut file = Box::new(FileImpl::new(ptr::addr_of_mut!(**fs_impl)));
        *root = ptr::addr_of_mut!(file.interface);
        fs_impl.open_files.push(file);

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
    // TODO
    let this_impl: *const FileImpl = this.cast();

    let mut file = Box::new(FileImpl::new((*this_impl).fs));
    *new_handle = ptr::addr_of_mut!(file.interface);
    (*(*this_impl).fs).open_files.push(file);

    Status::SUCCESS
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
