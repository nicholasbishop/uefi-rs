// TODO
#![allow(unused_variables)]

pub mod uefi_services;

mod boot;
mod console;
mod fs;
mod loaded_image;
mod runtime;
mod text;

use boot::{install_owned_protocol, open_protocol, SharedAnyBox, SharedBox};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use std::{mem, ptr};
use uefi::proto::console::text::Output;
use uefi::proto::device_path::build::DevicePathBuilder;
use uefi::proto::device_path::text::{DevicePathFromText, DevicePathToText};
use uefi::proto::device_path::DevicePath;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::{BootServices, MemoryType};
use uefi::table::runtime::RuntimeServices;
use uefi::table::{Boot, Header, Revision, SystemTable, SystemTableImpl};
use uefi::{CString16, Handle, Identify, Status};

#[macro_export]
macro_rules! try_status {
    ($expr:expr) => {
        match $expr {
            Status::SUCCESS => (),
            status => {
                return status;
            }
        }
    };
}

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
            install_protocol_interface,
            reinstall_protocol_interface,
            uninstall_protocol_interface,
            handle_protocol,
            _reserved: 0,
            register_protocol_notify,
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

    let stdout_handle = text::install_output_protocol().unwrap();

    let boot_fs_handle = {
        let mut buf = [MaybeUninit::uninit(); 256];
        // TODO: make a real path
        let path = DevicePathBuilder::with_buf(&mut buf).finalize().unwrap();

        // Wrap the DST device path in an intermediate struct so that we can store
        // it in a SharedAnyBox.
        #[repr(transparent)]
        struct DevicePathWrapper(SharedBox<DevicePath>);
        let mut data = SharedAnyBox::new(DevicePathWrapper(SharedBox::new(path)));
        let tmp = data.downcast_mut::<DevicePathWrapper>().unwrap();
        let interface = tmp.0.as_mut_ptr();

        install_owned_protocol(None, DevicePath::GUID, interface.cast(), data).unwrap()
    };

    fs::install_simple_file_system(boot_fs_handle).unwrap();

    let image = {
        let mut data = SharedAnyBox::new(LoadedImage {
            revision: 1,
            parent_handle: bad_handle,
            system_table: ptr::null(),
            device_handle: boot_fs_handle,
            file_path: ptr::null(),
            _reserved: ptr::null(),
            load_options_size: 0,
            load_options: ptr::null(),

            // Location where image was loaded
            image_base: ptr::null(),
            image_size: 0,
            image_code_type: MemoryType::LOADER_CODE,
            image_data_type: MemoryType::LOADER_DATA,
            unload: loaded_image::unload,

            _no_send_or_sync: PhantomData,
        });

        install_owned_protocol(None, LoadedImage::GUID, data.as_mut_ptr().cast(), data).unwrap()
    };

    {
        let mut data = SharedAnyBox::new(text::make_device_path_to_text());
        let interface = data.as_mut_ptr();

        install_owned_protocol(None, DevicePathToText::GUID, interface.cast(), data).unwrap();
    }

    {
        let mut data = SharedAnyBox::new(text::make_device_path_from_text());
        let interface = data.as_mut_ptr();

        install_owned_protocol(None, DevicePathFromText::GUID, interface.cast(), data).unwrap();
    }

    console::install_serial_protocol().unwrap();

    let mut stdout_ptr = ptr::null_mut();
    assert_eq!(
        open_protocol(
            stdout_handle,
            &Output::GUID,
            &mut stdout_ptr,
            image,
            None,
            0,
        ),
        Status::SUCCESS
    );

    let mut system_table_impl = SystemTableImpl {
        header: Header {
            signature: 0x1234_5678,
            revision: Revision::new(2, 90),
            size: 0,
            crc: 0,
            _reserved: 0,
        },
        fw_vendor: fw_vendor.as_ptr(),
        fw_revision: 0x1516_1718,
        stdin_handle: bad_handle,
        stdin: ptr::null_mut(),
        stdout_handle: stdout_handle,
        stdout: stdout_ptr.cast(),
        stderr_handle: bad_handle,
        stderr: ptr::null_mut(),
        // TODO
        runtime: unsafe { &*(&runtime_services as *const RuntimeServices) },
        boot: &mut boot_services,
        nr_cfg: 0,
        cfg_table: ptr::null(),
    };

    let st: SystemTable<Boot> = unsafe {
        SystemTable::from_ptr((&mut system_table_impl as *mut SystemTableImpl).cast()).unwrap()
    };

    entry(image, st)
}
