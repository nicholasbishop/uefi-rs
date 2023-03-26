//! Block I/O protocols.

use crate::proto::unsafe_protocol;
use crate::Status;

/// The Block I/O protocol.
#[repr(C)]
#[unsafe_protocol("964e5b21-6459-11d2-8e39-00a0c969723b")]
pub struct BlockIO {
    pub revision: u64,
    pub media: *const BlockIOMedia,

    pub reset: extern "efiapi" fn(this: &BlockIO, extended_verification: bool) -> Status,
    pub read_blocks: extern "efiapi" fn(
        this: &BlockIO,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *mut u8,
    ) -> Status,
    pub write_blocks: extern "efiapi" fn(
        this: &BlockIO,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *const u8,
    ) -> Status,
    pub flush_blocks: extern "efiapi" fn(this: &BlockIO) -> Status,
}

/// EFI LBA type
pub type Lba = u64;

/// Media information structure
#[repr(C)]
#[derive(Debug)]
pub struct BlockIOMedia {
    pub media_id: u32,
    pub removable_media: bool,
    pub media_present: bool,
    pub logical_partition: bool,
    pub read_only: bool,
    pub write_caching: bool,

    pub block_size: u32,
    pub io_align: u32,
    pub last_block: Lba,

    // Revision 2
    pub lowest_aligned_lba: Lba,
    pub logical_blocks_per_physical_block: u32,

    // Revision 3
    pub optimal_transfer_length_granularity: u32,
}
