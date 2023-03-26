//! File system support protocols.

use super::file::FileImpl;
use crate::{guid, Guid, Identify, Status};

#[repr(C)]
pub struct SimpleFileSystem {
    pub revision: u64,
    pub open_volume:
        extern "efiapi" fn(this: &mut SimpleFileSystem, root: &mut *mut FileImpl) -> Status,
}

unsafe impl Identify for SimpleFileSystem {
    const GUID: Guid = guid!("964e5b22-6459-11d2-8e39-00a0c969723b");
}
