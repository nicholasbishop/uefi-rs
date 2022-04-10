use crate::uefi_stub::STATE;
use core::ffi::c_void;
use core::slice;
use uefi::table::boot::MemoryDescriptor;
use uefi::{CStr16, CString16};
use uefi_raw::capsule::CapsuleHeader;
use uefi_raw::table::runtime::{ResetType, RuntimeServices, TimeCapabilities, VariableAttributes};
use uefi_raw::table::{Header, Revision};
use uefi_raw::time::Time;
use uefi_raw::{Char16, Guid, PhysicalAddress, Status};

pub type VariableKey = (Guid, CString16);
pub type VariableData = (VariableAttributes, Vec<u8>);

unsafe extern "efiapi" fn get_time(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status {
    todo!()
}

unsafe extern "efiapi" fn set_time(time: *const Time) -> Status {
    todo!()
}

unsafe extern "efiapi" fn get_wakeup_time(
    enabled: *mut u8,
    pending: *mut u8,
    time: *mut Time,
) -> Status {
    todo!()
}

unsafe extern "efiapi" fn set_wakeup_time(enable: u8, time: *const Time) -> Status {
    todo!()
}

unsafe extern "efiapi" fn set_virtual_address_map(
    map_size: usize,
    desc_size: usize,
    desc_version: u32,
    virtual_map: *mut MemoryDescriptor,
) -> Status {
    todo!()
}

unsafe extern "efiapi" fn convert_pointer(
    debug_disposition: usize,
    address: *mut *const c_void,
) -> Status {
    todo!()
}

unsafe extern "efiapi" fn get_variable(
    variable_name: *const Char16,
    vendor_guid: *const Guid,
    attributes: *mut VariableAttributes,
    data_size: *mut usize,
    data: *mut u8,
) -> Status {
    STATE.with(|state| {
        let state = state.borrow_mut();

        let name = CStr16::from_ptr(variable_name.cast());
        let key = (*vendor_guid, name.to_owned());
        if let Some((src_attr, src_data)) = state.variables.get(&key) {
            if *data_size < src_data.len() {
                *data_size = src_data.len();
                Status::BUFFER_TOO_SMALL
            } else {
                *attributes = *src_attr;
                *data_size = src_data.len();
                let dst_data = slice::from_raw_parts_mut(data, *data_size);
                dst_data.copy_from_slice(src_data);
                Status::SUCCESS
            }
        } else {
            Status::NOT_FOUND
        }
    })
}

unsafe extern "efiapi" fn get_next_variable_name(
    variable_name_size: *mut usize,
    variable_name: *mut u16,
    vendor_guid: *mut Guid,
) -> Status {
    // TODO
    Status::NOT_FOUND
}

unsafe extern "efiapi" fn set_variable(
    variable_name: *const Char16,
    vendor_guid: *const Guid,
    attributes: VariableAttributes,
    data_size: usize,
    data: *const u8,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let name = CStr16::from_ptr(variable_name.cast());
        let key = (*vendor_guid, name.to_owned());

        if data_size == 0 {
            state.variables.remove(&key);
        } else {
            let data = slice::from_raw_parts(data, data_size);
            let value = (attributes, data.to_vec());
            state.variables.insert(key, value);
        }

        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn get_next_high_monotonic_count(high_count: *mut u32) -> Status {
    todo!()
}

unsafe extern "efiapi" fn reset_system(
    rt: ResetType,
    status: Status,
    data_size: usize,
    data: *const u8,
) -> ! {
    std::process::exit(status.0.try_into().unwrap())
}

unsafe extern "efiapi" fn update_capsule(
    capsule_header_array: *const *const CapsuleHeader,
    capsule_count: usize,
    scatter_gather_list: PhysicalAddress,
) -> Status {
    todo!()
}

unsafe extern "efiapi" fn query_capsule_capabilities(
    capsule_header_array: *const *const CapsuleHeader,
    capsule_count: usize,
    maximum_capsule_size: *mut usize,
    reset_type: *mut ResetType,
) -> Status {
    todo!()
}

// Miscellaneous UEFI 2.0 Service.
unsafe extern "efiapi" fn query_variable_info(
    attributes: VariableAttributes,
    maximum_variable_storage_size: *mut u64,
    remaining_variable_storage_size: *mut u64,
    maximum_variable_size: *mut u64,
) -> Status {
    Status::SUCCESS
}

pub fn new_runtime_services() -> RuntimeServices {
    RuntimeServices {
        // TODO
        header: Header {
            revision: Revision::EFI_2_100,
            ..Default::default()
        },

        get_time,
        set_time,
        get_wakeup_time,
        set_wakeup_time,
        set_virtual_address_map,
        convert_pointer,
        get_variable,
        get_next_variable_name,
        set_variable,
        get_next_high_monotonic_count,
        reset_system,
        update_capsule,
        query_capsule_capabilities,
        query_variable_info,
    }
}
