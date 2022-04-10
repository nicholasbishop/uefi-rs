// TODO
#![allow(unused_variables)]

pub mod uefi_services;

mod boot;
mod logger;
mod proto;
mod runtime;
mod shared_box;

use boot::{
    install_owned_protocol, open_protocol, with_owned_protocol_data, EventImpl, HandleImpl, Pages,
};
use core::mem::MaybeUninit;
use proto::fs::FsDb;
use runtime::{VariableData, VariableKey};
use shared_box::{SharedAnyBox, SharedBox};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ptr;
use std::rc::Rc;
use uefi::proto::console::text::Output;
use uefi::proto::device_path::build::{self, DevicePathBuilder};
use uefi::proto::device_path::media::{PartitionFormat, PartitionSignature};
use uefi::proto::device_path::text::{DevicePathFromText, DevicePathToText};
use uefi::proto::device_path::DevicePath;
use uefi::table::Revision;
use uefi::{cstr16, CString16, Identify, Status};
use uefi_raw::protocol::console::SimpleTextInputProtocol;
use uefi_raw::protocol::loaded_image::LoadedImageProtocol;
use uefi_raw::table::boot::{BootServices, MemoryAttribute, MemoryDescriptor, MemoryType};
use uefi_raw::table::configuration::ConfigurationTable;
use uefi_raw::table::runtime::RuntimeServices;
use uefi_raw::table::system::SystemTable;
use uefi_raw::table::Header;
use uefi_raw::{Event, Handle};

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

pub struct State {
    // TODO: de-option?
    system_table: Option<SharedBox<SystemTable>>,
    boot_services: Option<SharedBox<BootServices>>,
    runtime_services: Option<SharedBox<RuntimeServices>>,
    configuration_tables: Vec<ConfigurationTable>,

    // Boot state.
    handle_db: HashMap<Handle, Box<HandleImpl>>,
    events: HashMap<Event, Box<EventImpl>>,
    pages: Vec<Pages>,
    memory_descriptors: Vec<MemoryDescriptor>,
    fs_db: FsDb,
    images: Vec<Box<boot::ImageImpl>>,

    // Runtime state.
    variables: BTreeMap<VariableKey, VariableData>,
}

// All "global" state goes in this thread local block. UEFI is single
// threaded, so we have types like `Protocols` that can't be shared
// between threads.
thread_local! {
    pub static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State {
        system_table: None,
        boot_services: None,
        runtime_services: None,
        configuration_tables: Vec::new(),

        handle_db: HashMap::new(),
        events: HashMap::new(),
        pages: Vec::new(),
        // Stub in some data to get past the memory test.
        memory_descriptors: vec![MemoryDescriptor {
            ty: MemoryType::LOADER_CODE,
            phys_start: 0,
            virt_start: 0,
            page_count: 1,
            att: MemoryAttribute::empty(),
        }],
        fs_db: FsDb::default(),
        images: Vec::new(),
        variables: BTreeMap::new(),
    }));
}

pub fn launch<E>(entry: E) -> Status
where
    E: Fn(Handle, *mut SystemTable) -> Status,
{
    let mut runtime_services = SharedBox::new(&runtime::new_runtime_services());
    let mut boot_services = SharedBox::new(&boot::new_boot_services());

    let fw_vendor = CString16::try_from("uefi_stub").unwrap();

    let stdout_handle = proto::text::install_output_protocol().unwrap();
    let stderr_handle = proto::text::install_output_protocol().unwrap();
    let stdin_handle = proto::text::install_input_protocol().unwrap();

    let boot_fs_handle = {
        let mut buf = [MaybeUninit::uninit(); 256];
        // TODO: make a real path
        let path = DevicePathBuilder::with_buf(&mut buf).finalize().unwrap();

        // Wrap the DST device path in an intermediate struct so that we can store
        // it in a SharedAnyBox.
        #[repr(transparent)]
        struct DevicePathWrapper(SharedBox<DevicePath>);
        let mut interface = SharedAnyBox::new(DevicePathWrapper(SharedBox::new(path)));
        let tmp = interface.downcast_mut::<DevicePathWrapper>().unwrap();
        let interface_ptr = tmp.0.as_mut_ptr();

        install_owned_protocol(
            None,
            DevicePath::GUID,
            interface_ptr.cast(),
            interface,
            None,
        )
        .unwrap()
    };

    proto::fs::install_simple_file_system(boot_fs_handle).unwrap();

    let mut image = ptr::null_mut();
    {
        // TODO: dedup this device path stuff with the other location.
        let mut buf = [MaybeUninit::uninit(); 256];
        let path = DevicePathBuilder::with_buf(&mut buf)
            .push(&build::acpi::Acpi {
                hid: 0x41d0_0a03,
                uid: 0,
            })
            .unwrap()
            .push(&build::hardware::Pci {
                device: 0x1f,
                function: 0x02,
            })
            .unwrap()
            .push(&build::messaging::Sata {
                hba_port_number: 0x0,
                port_multiplier_port_number: 0xffff,
                logical_unit_number: 0x0,
            })
            .unwrap()
            .push(&build::media::HardDrive {
                partition_number: 1,
                partition_start: 0x3f,
                partition_size: 0xfbfc1,
                partition_signature: PartitionSignature::Mbr(0xbe1afdfau32.to_le_bytes()),
                partition_format: PartitionFormat::MBR,
            })
            .unwrap()
            .push(&build::media::FilePath {
                path_name: cstr16!("\\efi\\boot\\test_runner.efi"),
            })
            .unwrap()
            .finalize()
            .unwrap();
        let boot_policy = 1;
        let parent_image_handle = ptr::null_mut();
        let source_buf = ptr::null();
        let source_len = 0;
        assert_eq!(
            unsafe {
                boot::load_image(
                    boot_policy,
                    parent_image_handle,
                    path.as_ffi_ptr().cast(),
                    source_buf,
                    source_len,
                    &mut image,
                )
            },
            Status::SUCCESS
        );

        let agent = image;
        let controller = ptr::null_mut();
        let attributes = 0x00000002;
        let mut loaded_image = ptr::null_mut();
        assert_eq!(
            open_protocol(
                image,
                &LoadedImageProtocol::GUID,
                &mut loaded_image,
                agent,
                controller,
                attributes,
            ),
            Status::SUCCESS
        );
        let loaded_image: *mut LoadedImageProtocol = loaded_image.cast();
        unsafe {
            (*loaded_image).device_handle = boot_fs_handle;
        }
        assert_eq!(
            boot::close_protocol(image, &LoadedImageProtocol::GUID, agent, controller,),
            Status::SUCCESS
        );
    }

    {
        let mut interface = SharedAnyBox::new(proto::device_path_text::make_device_path_to_text());
        install_owned_protocol(
            None,
            DevicePathToText::GUID,
            interface.as_mut_ptr().cast(),
            interface,
            None,
        )
        .unwrap();
    }

    {
        let mut interface =
            SharedAnyBox::new(proto::device_path_text::make_device_path_from_text());
        install_owned_protocol(
            None,
            DevicePathFromText::GUID,
            interface.as_mut_ptr().cast(),
            interface,
            None,
        )
        .unwrap();
    }

    proto::console::install_serial_protocol().unwrap();
    proto::gop::install_gop_protocol().unwrap();
    proto::pointer::install_pointer_protocol().unwrap();

    let mut stdout_ptr = ptr::null_mut();
    assert_eq!(
        open_protocol(
            stdout_handle,
            &Output::GUID,
            &mut stdout_ptr,
            image,
            ptr::null_mut(),
            0,
        ),
        Status::SUCCESS
    );

    let mut stderr_ptr = ptr::null_mut();
    assert_eq!(
        open_protocol(
            stderr_handle,
            &Output::GUID,
            &mut stderr_ptr,
            image,
            ptr::null_mut(),
            0,
        ),
        Status::SUCCESS
    );

    let mut stdin_ptr = ptr::null_mut();
    assert_eq!(
        open_protocol(
            stdin_handle,
            &SimpleTextInputProtocol::GUID,
            &mut stdin_ptr,
            image,
            ptr::null_mut(),
            0,
        ),
        Status::SUCCESS
    );

    let system_table: *mut SystemTable = STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.system_table = Some(SharedBox::new(&SystemTable {
            header: Header {
                signature: 0x1234_5678,
                revision: Revision::new(2, 90),
                size: 0,
                crc: 0,
                reserved: 0,
            },

            firmware_vendor: fw_vendor.as_ptr().cast(),
            firmware_revision: 0x1516_1718,

            stdin_handle,
            stdin: stdin_ptr.cast(),

            stdout_handle,
            stdout: stdout_ptr.cast(),

            stderr_handle,
            stderr: stderr_ptr.cast(),

            runtime_services: runtime_services.as_mut_ptr(),
            boot_services: boot_services.as_mut_ptr(),

            number_of_configuration_table_entries: 0,
            configuration_table: ptr::null_mut(),
        }));
        state.boot_services = Some(boot_services);
        state.runtime_services = Some(runtime_services);
        state.system_table.as_mut().unwrap().as_mut_ptr()
    });

    entry(image, system_table)
}
