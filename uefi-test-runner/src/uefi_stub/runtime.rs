// TODO
#![allow(unused_variables)]

use uefi::table::boot::MemoryDescriptor;
use uefi::table::runtime::{ResetType, Time, TimeCapabilities, VariableAttributes};
use uefi::{Char16, Guid, Status};

pub unsafe extern "efiapi" fn get_time(
    time: *mut Time,
    capabilities: *mut TimeCapabilities,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn set_time(time: &Time) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn set_virtual_address_map(
    map_size: usize,
    desc_size: usize,
    desc_version: u32,
    virtual_map: *mut MemoryDescriptor,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn get_variable(
    variable_name: *const Char16,
    vendor_guid: *const Guid,
    attributes: *mut VariableAttributes,
    data_size: *mut usize,
    data: *mut u8,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn get_next_variable_name(
    variable_name_size: *mut usize,
    variable_name: *mut u16,
    vendor_guid: *mut Guid,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn set_variable(
    variable_name: *const Char16,
    vendor_guid: *const Guid,
    attributes: VariableAttributes,
    data_size: usize,
    data: *const u8,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn reset(
    rt: ResetType,

    status: Status,
    data_size: usize,
    data: *const u8,
) -> ! {
    todo!()
}

// Miscellaneous UEFI 2.0 Service.
pub unsafe extern "efiapi" fn query_variable_info(
    attributes: VariableAttributes,
    maximum_variable_storage_size: *mut u64,
    remaining_variable_storage_size: *mut u64,
    maximum_variable_size: *mut u64,
) -> Status {
    todo!()
}
