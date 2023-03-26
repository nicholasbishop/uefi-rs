//! Disk I/O protocols.

use crate::{guid, Event, Guid, Status};
use core::ptr::NonNull;

/// The disk I/O protocol.
///
/// This protocol is used to abstract the block accesses of the block I/O
/// protocol to a more general offset-length protocol. Firmware is
/// responsible for adding this protocol to any block I/O interface that
/// appears in the system that does not already have a disk I/O protocol.
#[repr(C)]
pub struct DiskIo {
    pub revision: u64,
    pub read_disk: extern "efiapi" fn(
        this: &DiskIo,
        media_id: u32,
        offset: u64,
        len: usize,
        buffer: *mut u8,
    ) -> Status,
    pub write_disk: extern "efiapi" fn(
        this: &mut DiskIo,
        media_id: u32,
        offset: u64,
        len: usize,
        buffer: *const u8,
    ) -> Status,
}

impl DiskIo {
    pub const GUID: Guid = guid!("ce345171-ba0b-11d2-8e4f-00a0c969723b");
}

/// Asynchronous transaction token for disk I/O 2 operations.
#[repr(C)]
#[derive(Debug)]
pub struct DiskIo2Token {
    /// Event to be signalled when an asynchronous disk I/O operation completes.
    pub event: Option<Event>,
    /// Transaction status code.
    pub transaction_status: Status,
}

/// The disk I/O 2 protocol.
///
/// This protocol provides an extension to the disk I/O protocol to enable
/// non-blocking / asynchronous byte-oriented disk operation.
#[repr(C)]
pub struct DiskIo2 {
    pub revision: u64,
    pub cancel: extern "efiapi" fn(this: &mut DiskIo2) -> Status,
    pub read_disk_ex: extern "efiapi" fn(
        this: &DiskIo2,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *mut u8,
    ) -> Status,
    pub write_disk_ex: extern "efiapi" fn(
        this: &mut DiskIo2,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *const u8,
    ) -> Status,
    pub flush_disk_ex:
        extern "efiapi" fn(this: &mut DiskIo2, token: Option<NonNull<DiskIo2Token>>) -> Status,
}

impl DiskIo2 {
    pub const GUID: Guid = guid!("151c8eae-7f2c-472c-9e54-9828194f6a88");
}
