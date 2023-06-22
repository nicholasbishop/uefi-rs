use crate::{guid, Char16, Guid, Status};
use core::ffi::c_void;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FileProtocol {
    pub revision: u64,
    pub open: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_handle: *mut *mut Self,
        filename: *const Char16,
        open_mode: u64,
        attributes: u64,
    ) -> Status,
    pub close: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub delete: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub read: unsafe extern "efiapi" fn(
        this: *mut Self,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write: unsafe extern "efiapi" fn(
        this: *mut Self,
        buffer_size: *mut usize,
        buffer: *const c_void,
    ) -> Status,
    pub get_position: unsafe extern "efiapi" fn(this: *mut Self, position: *mut u64) -> Status,
    pub set_position: unsafe extern "efiapi" fn(this: *mut Self, position: u64) -> Status,
    pub get_info: unsafe extern "efiapi" fn(
        this: *mut Self,
        information_type: *const Guid,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub set_info: unsafe extern "efiapi" fn(
        this: *mut Self,
        information_type: *const Guid,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
    pub flush: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume:
        unsafe extern "efiapi" fn(this: *mut Self, root: *mut *mut FileProtocol) -> Status,
}

impl SimpleFileSystemProtocol {
    pub const GUID: Guid = guid!("964e5b22-6459-11d2-8e39-00a0c969723b");
}
