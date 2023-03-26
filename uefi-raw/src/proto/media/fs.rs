//! File system support protocols.

use super::file::FileImpl;
use crate::proto::unsafe_protocol;
use crate::Status;

#[repr(C)]
#[unsafe_protocol("964e5b22-6459-11d2-8e39-00a0c969723b")]
pub struct SimpleFileSystem {
    pub revision: u64,
    pub open_volume:
        extern "efiapi" fn(this: &mut SimpleFileSystem, root: &mut *mut FileImpl) -> Status,
}
