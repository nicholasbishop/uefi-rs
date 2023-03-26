//! Block I/O protocols.

use crate::proto::unsafe_protocol;
use crate::Status;

/// The Block I/O protocol.
#[repr(C)]
#[unsafe_protocol("964e5b21-6459-11d2-8e39-00a0c969723b")]
pub struct BlockIO {
    revision: u64,
    media: *const BlockIOMedia,

    reset: extern "efiapi" fn(this: &BlockIO, extended_verification: bool) -> Status,
    read_blocks: extern "efiapi" fn(
        this: &BlockIO,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *mut u8,
    ) -> Status,
    write_blocks: extern "efiapi" fn(
        this: &BlockIO,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *const u8,
    ) -> Status,
    flush_blocks: extern "efiapi" fn(this: &BlockIO) -> Status,
}

/// EFI LBA type
pub type Lba = u64;

/// Media information structure
#[repr(C)]
#[derive(Debug)]
pub struct BlockIOMedia {
    media_id: u32,
    removable_media: bool,
    media_present: bool,
    logical_partition: bool,
    read_only: bool,
    write_caching: bool,

    block_size: u32,
    io_align: u32,
    last_block: Lba,

    // Revision 2
    lowest_aligned_lba: Lba,
    logical_blocks_per_physical_block: u32,

    // Revision 3
    optimal_transfer_length_granularity: u32,
}

impl BlockIOMedia {
    /// The current media ID.
    #[must_use]
    pub const fn media_id(&self) -> u32 {
        self.media_id
    }

    /// True if the media is removable.
    #[must_use]
    pub const fn is_removable_media(&self) -> bool {
        self.removable_media
    }

    /// True if there is a media currently present in the device.
    #[must_use]
    pub const fn is_media_present(&self) -> bool {
        self.media_present
    }

    /// True if block IO was produced to abstract partition structure.
    #[must_use]
    pub const fn is_logical_partition(&self) -> bool {
        self.logical_partition
    }

    /// True if the media is marked read-only.
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// True if `writeBlocks` function writes data.
    #[must_use]
    pub const fn is_write_caching(&self) -> bool {
        self.write_caching
    }

    /// The intrinsic block size of the device.
    ///
    /// If the media changes, then this field is updated. Returns the number of bytes per logical block.
    #[must_use]
    pub const fn block_size(&self) -> u32 {
        self.block_size
    }

    /// Supplies the alignment requirement for any buffer used in a data transfer.
    #[must_use]
    pub const fn io_align(&self) -> u32 {
        self.io_align
    }

    /// The last LBA on the device. If the media changes, then this field is updated.
    #[must_use]
    pub const fn last_block(&self) -> Lba {
        self.last_block
    }

    /// Returns the first LBA that is aligned to a physical block boundary.
    #[must_use]
    pub const fn lowest_aligned_lba(&self) -> Lba {
        self.lowest_aligned_lba
    }

    /// Returns the number of logical blocks per physical block.
    #[must_use]
    pub const fn logical_blocks_per_physical_block(&self) -> u32 {
        self.logical_blocks_per_physical_block
    }

    /// Returns the optimal transfer length granularity as a number of logical blocks.
    #[must_use]
    pub const fn optimal_transfer_length_granularity(&self) -> u32 {
        self.optimal_transfer_length_granularity
    }
}
