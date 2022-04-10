// TODO
#![allow(unused_variables)]

pub mod uefi_services;

mod runtime;
mod text;

use core::{mem, ptr};
use uefi::proto::console::text::{Output, OutputData};
use uefi::table::runtime::RuntimeServices;
use uefi::table::{Boot, Header, Revision, SystemTable, SystemTableImpl};
use uefi::{CString16, Handle, Status};

pub fn launch<E>(entry: E) -> Status
where
    E: Fn(Handle, SystemTable<Boot>) -> Status,
{
    // TODO
    use runtime::*;
    let bad_handle: Handle = unsafe { mem::transmute(0xbad_badu64) };

    let runtime_services = RuntimeServices {
        header: Header {
            signature: 0x1234_5678,
            revision: Revision::new(0, 1),
            size: 0,
            crc: 0,
            _reserved: 0,
        },
        get_time,
        set_time,
        _pad: [0; 2],
        set_virtual_address_map,
        _pad2: 0,
        get_variable,
        get_next_variable_name,
        set_variable,
        _pad3: 0,
        reset,
        update_capsule: 0,
        query_capsule_capabilities: 0,
        query_variable_info,
    };

    let fw_vendor = CString16::try_from("uefi_stub").unwrap();

    let output_data = OutputData {
        max_mode: 0,
        mode: 0,
        attribute: 0,
        cursor_column: 0,
        cursor_row: 0,
        cursor_visible: false,
    };

    let mut stdout = Output {
        reset: text::reset,
        output_string: text::output_string,
        test_string: text::test_string,
        query_mode: text::query_mode,
        set_mode: text::set_mode,
        set_attribute: text::set_attribute,
        clear_screen: text::clear_screen,
        set_cursor_position: text::set_cursor_position,
        enable_cursor: text::enable_cursor,
        // TODO
        data: unsafe { &*(&output_data as *const OutputData) },
    };

    let mut system_table_impl = SystemTableImpl {
        header: Header {
            signature: 0x1234_5678,
            revision: Revision::new(0, 1),
            size: 0,
            crc: 0,
            _reserved: 0,
        },
        fw_vendor: fw_vendor.as_ptr(),
        fw_revision: Revision::new(1, 2),
        stdin_handle: bad_handle,
        stdin: ptr::null_mut(),
        stdout_handle: bad_handle,
        stdout: &mut stdout,
        stderr_handle: bad_handle,
        stderr: ptr::null_mut(),
        // TODO
        runtime: unsafe { &*(&runtime_services as *const RuntimeServices) },
        boot: ptr::null(),
        nr_cfg: 0,
        cfg_table: ptr::null(),
    };

    //let st: SystemTable<Boot> = uefi_stub::create_system_table();
    let st: SystemTable<Boot> = unsafe {
        SystemTable::from_ptr((&mut system_table_impl as *mut SystemTableImpl).cast()).unwrap()
    };

    entry(bad_handle, st)
}
