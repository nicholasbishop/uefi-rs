use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::slice;
use uefi::table::boot::MemoryDescriptor;
use uefi::table::runtime::{ResetType, Time, TimeCapabilities, VariableAttributes};
use uefi::{CStr16, CString16, Char16, Guid, Status};

type VariableKey = (Guid, CString16);
type VariableData = (VariableAttributes, Vec<u8>);

pub struct State {
    variables: BTreeMap<VariableKey, VariableData>,
}

// TODO: this follows the pattern in the boot.rs stub.
thread_local! {
    pub static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State {
        variables: BTreeMap::new(),
    }))
}

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
    STATE.with(|state| {
        let state = state.borrow_mut();

        let name = CStr16::from_ptr(variable_name);
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

pub unsafe extern "efiapi" fn get_next_variable_name(
    variable_name_size: *mut usize,
    variable_name: *mut u16,
    vendor_guid: *mut Guid,
) -> Status {
    // TODO
    Status::NOT_FOUND
}

pub unsafe extern "efiapi" fn set_variable(
    variable_name: *const Char16,
    vendor_guid: *const Guid,
    attributes: VariableAttributes,
    data_size: usize,
    data: *const u8,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let name = CStr16::from_ptr(variable_name);
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
