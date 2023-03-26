use crate::table::boot::MemoryAttribute;
use crate::{guid, Guid, PhysicalAddress, Status};

/// Protocol for getting and setting memory protection attributes.
///
/// Corresponds to the C type `EFI_MEMORY_ATTRIBUTE_PROTOCOL`.
#[repr(C)]
pub struct MemoryProtection {
    pub get_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: *mut MemoryAttribute,
    ) -> Status,

    pub set_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: MemoryAttribute,
    ) -> Status,

    pub clear_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: MemoryAttribute,
    ) -> Status,
}

impl MemoryProtection {
    pub const GUID: Guid = guid!("f4560cf6-40ec-4b4a-a192-bf1d57d0b189");
}
