use super::boot::STATE;
use crate::uefi_stub::{install_owned_protocol, SharedAnyBox};
use core::marker::PhantomData;
use std::collections::HashMap;
use std::ffi::c_void;
use std::{mem, ptr};
use uefi::proto::media::file::{FileAttribute, FileImpl, FileInfo, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::runtime::Time;
use uefi::{CString16, Char16, Guid, Handle, Identify, Result, Status};

pub struct FsImpl {
    root: FileImpl,
}

pub type FsDb = HashMap<*const SimpleFileSystem, Box<FsImpl>>;

pub fn install_simple_file_system(handle: Handle) -> Result {
    let mut interface = SharedAnyBox::new(SimpleFileSystem {
        revision: 0,
        open_volume,
        _no_send_or_sync: PhantomData,
    });
    let sfs = interface.as_mut_ptr();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        // TODO: move fs_db into protocol owned data?
        state.fs_db.insert(
            sfs.cast(),
            Box::new(FsImpl {
                root: FileImpl {
                    revision: 0,
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
            }),
        );
    });

    install_owned_protocol(
        Some(handle),
        SimpleFileSystem::GUID,
        sfs.cast(),
        interface,
        None,
    )?;

    Ok(())
}

unsafe extern "efiapi" fn open_volume(
    this: *mut SimpleFileSystem,
    root: &mut *mut FileImpl,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let this: *const SimpleFileSystem = this;
        let fs_impl = state.fs_db.get_mut(&this).unwrap();
        *root = &mut fs_impl.root;

        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn open(
    this: &mut FileImpl,
    new_handle: &mut *mut FileImpl,
    filename: *const Char16,
    open_mode: FileMode,
    attributes: FileAttribute,
) -> Status {
    todo!()
}

extern "efiapi" fn close(this: &mut FileImpl) -> Status {
    todo!()
}

extern "efiapi" fn delete(this: &mut FileImpl) -> Status {
    todo!()
}

unsafe extern "efiapi" fn read(
    this: &mut FileImpl,
    buffer_size: &mut usize,
    buffer: *mut u8,
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
    ptr::copy(info_ptr.cast::<u8>(), buffer, required_size);

    Status::SUCCESS
}

unsafe extern "efiapi" fn write(
    this: &mut FileImpl,
    buffer_size: &mut usize,
    buffer: *const u8,
) -> Status {
    todo!()
}

extern "efiapi" fn get_position(this: &mut FileImpl, position: &mut u64) -> Status {
    todo!()
}

extern "efiapi" fn set_position(this: &mut FileImpl, position: u64) -> Status {
    todo!()
}

unsafe extern "efiapi" fn get_info(
    this: &mut FileImpl,
    information_type: &Guid,
    buffer_size: &mut usize,
    buffer: *mut u8,
) -> Status {
    todo!()
}

unsafe extern "efiapi" fn set_info(
    this: &mut FileImpl,
    information_type: &Guid,
    buffer_size: usize,
    buffer: *const c_void,
) -> Status {
    todo!()
}

extern "efiapi" fn flush(this: &mut FileImpl) -> Status {
    todo!()
}
