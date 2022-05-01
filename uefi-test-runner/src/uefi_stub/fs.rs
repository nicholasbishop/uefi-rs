use std::ffi::c_void;
use std::ptr;
use uefi::proto::media::file::{FileAttribute, FileImpl, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::{Char16, Guid, Status};

#[repr(C)]
struct VolumeFileImpl {
    file: FileImpl,
}

#[repr(C)]
pub struct SimpleFileSystemImpl {
    sfs: SimpleFileSystem,
    volume: VolumeFileImpl,
}

impl SimpleFileSystemImpl {
    pub fn new() -> Self {
        Self {
            sfs: SimpleFileSystem {
                revision: 0,
                open_volume,
            },
            volume: VolumeFileImpl {
                file: FileImpl {
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
            },
        }
    }
}

extern "efiapi" fn open_volume(this: *mut SimpleFileSystem, root: &mut *mut FileImpl) -> Status {
    let this = this.cast::<SimpleFileSystemImpl>();
    *root = unsafe { ptr::addr_of_mut!((*this).volume.file) };
    Status::SUCCESS
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
    todo!()
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
