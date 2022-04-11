// TODO
#![allow(unused_variables)]

pub mod uefi_services;

mod boot;
mod runtime;
mod text;

use boot::install_protocol;
use core::{mem, ptr};
use uefi::proto::console::text::{Output, OutputData};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::{BootServices, MemoryType};
use uefi::table::runtime::RuntimeServices;
use uefi::table::{Boot, Header, Revision, SystemTable, SystemTableImpl};
use uefi::{CString16, Handle, Identify, Status};

pub fn launch<E>(entry: E) -> Status
where
    E: Fn(Handle, SystemTable<Boot>) -> Status,
{
    // TODO
    let bad_handle: Handle = unsafe { mem::transmute(0xbad_badu64) };

    let runtime_services = {
        use runtime::*;
        RuntimeServices {
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
        }
    };

    let mut boot_services = {
        use boot::*;
        BootServices {
            header: Header {
                signature: 0x1234_5678,
                revision: Revision::new(0, 1),
                size: 0,
                crc: 0,
                _reserved: 0,
            },
            raise_tpl,
            restore_tpl,
            allocate_pages,
            free_pages,
            get_memory_map,
            allocate_pool,
            free_pool,
            create_event,
            set_timer,
            wait_for_event,
            signal_event,
            close_event,
            check_event,
            install_protocol_interface: 0,
            reinstall_protocol_interface: 0,
            uninstall_protocol_interface: 0,
            handle_protocol,
            _reserved: 0,
            register_protocol_notify: 0,
            locate_handle,
            locate_device_path,
            install_configuration_table: 0,
            load_image,
            start_image,
            exit,
            unload_image,
            exit_boot_services,
            get_next_monotonic_count: 0,
            stall,
            set_watchdog_timer,
            connect_controller,
            disconnect_controller,
            open_protocol,
            close_protocol,
            open_protocol_information: 0,
            protocols_per_handle,
            locate_handle_buffer,
            locate_protocol,
            install_multiple_protocol_interfaces: 0,
            uninstall_multiple_protocol_interfaces: 0,
            calculate_crc32: 0,
            copy_mem,
            set_mem,
            create_event_ex,
        }
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

    let mut stdout = {
        use text::*;
        Output {
            reset,
            output_string,
            test_string,
            query_mode,
            set_mode,
            set_attribute,
            clear_screen,
            set_cursor_position,
            enable_cursor,
            // TODO
            data: unsafe { &*(&output_data as *const OutputData) },
        }
    };

    let mut system_table_impl = SystemTableImpl {
        header: Header {
            signature: 0x1234_5678,
            revision: Revision::new(2, 90),
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
        boot: &mut boot_services,
        nr_cfg: 0,
        cfg_table: ptr::null(),
    };

    //let st: SystemTable<Boot> = uefi_stub::create_system_table();
    let st: SystemTable<Boot> = unsafe {
        SystemTable::from_ptr((&mut system_table_impl as *mut SystemTableImpl).cast()).unwrap()
    };

    let image = install_protocol(
        None,
        LoadedImage::GUID,
        Box::new(LoadedImage {
            revision: 1,
            parent_handle: bad_handle,
            system_table: ptr::null(),
            device_handle: bad_handle,
            file_path: ptr::null(),
            _reserved: ptr::null(),
            load_options_size: 0,
            load_options: ptr::null(),

            // Location where image was loaded
            image_base: ptr::null(),
            image_size: 0,
            image_code_type: MemoryType::LOADER_CODE,
            image_data_type: MemoryType::LOADER_DATA,
            unload: unsafe { mem::transmute(ptr::null::<()>()) },
        }),
    )
    .unwrap();

    entry(image, st)
}
