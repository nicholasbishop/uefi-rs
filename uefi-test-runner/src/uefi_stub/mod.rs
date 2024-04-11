// TODO
#![allow(unused_variables)]

pub mod uefi_services;

mod boot;
mod proto;
mod runtime;

use boot::{install_protocol_simple, open_protocol, EventImpl, HandleImpl, Pages};
use core::mem::MaybeUninit;
use core::pin::Pin;
use proto::fs::FsDb;
use runtime::{VariableData, VariableKey};
use std::cell::{RefCell, UnsafeCell};
use std::collections::{BTreeMap, HashMap};
use std::ptr::{self, addr_of_mut};
use std::rc::Rc;
use uefi::proto::device_path::build::{self, DevicePathBuilder};
use uefi::proto::device_path::media::{PartitionFormat, PartitionSignature};
use uefi::proto::device_path::DevicePath;
use uefi::{cstr16, CString16, Identify, Status};
use uefi_raw::protocol::console::{SimpleTextInputProtocol, SimpleTextOutputProtocol};
use uefi_raw::protocol::loaded_image::LoadedImageProtocol;
use uefi_raw::table::boot::{BootServices, MemoryAttribute, MemoryDescriptor, MemoryType};
use uefi_raw::table::configuration::ConfigurationTable;
use uefi_raw::table::runtime::RuntimeServices;
use uefi_raw::table::system::SystemTable;
use uefi_raw::table::{Header, Revision};
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
    system_table: UnsafeCell<SystemTable>,
    boot_services: BootServices,
    runtime_services: RuntimeServices,
    configuration_tables: Vec<ConfigurationTable>,

    // Boot state.
    handle_db: Vec<Pin<Box<HandleImpl>>>,
    events: HashMap<Event, Box<EventImpl>>,
    pages: Vec<Pages>,
    memory_descriptors: Vec<MemoryDescriptor>,
    fs_db: FsDb,
    images: Vec<Pin<Box<boot::ImageImpl>>>,

    // Runtime state.
    variables: BTreeMap<VariableKey, VariableData>,
}

impl State {
    fn find_handle(&self, handle: Handle) -> Option<&HandleImpl> {
        self.handle_db
            .iter()
            .map(|hi| &**hi)
            .find(|hi| hi.handle() == handle)
    }

    fn find_handle_mut(&mut self, handle: Handle) -> Option<&mut HandleImpl> {
        self.handle_db
            .iter_mut()
            .map(|hi| &mut **hi)
            .find(|hi| hi.handle() == handle)
    }
}

// All "global" state goes in this thread local block. UEFI is single
// threaded, so we have types like `Protocols` that can't be shared
// between threads.
thread_local! {
    pub static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State {
        system_table: UnsafeCell::new(SystemTable::default()),
        boot_services: boot::new_boot_services(),
        runtime_services: runtime::new_runtime_services(),
        configuration_tables: Vec::new(),

        handle_db: Vec::new(),
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
    let fw_vendor = CString16::try_from("uefi_stub").unwrap();

    let stdout_handle = proto::text::install_output_protocol().unwrap();
    let stderr_handle = proto::text::install_output_protocol().unwrap();
    let stdin_handle = proto::text::install_input_protocol().unwrap();

    let boot_fs_handle = {
        let buf = Box::new([MaybeUninit::uninit(); 256]);
        // TODO: leak.
        let buf = Box::leak(buf);
        let path = DevicePathBuilder::with_buf(buf).finalize().unwrap();

        let interface = path.as_ffi_ptr().cast();

        install_protocol_simple(None, &DevicePath::GUID, interface).unwrap()
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

    proto::device_path_text::install().unwrap();
    proto::console::install_serial_protocol().unwrap();
    proto::gop::install_gop_protocol().unwrap();
    proto::pointer::install_pointer_protocol().unwrap();

    let mut stdout_ptr = ptr::null_mut();
    assert_eq!(
        open_protocol(
            stdout_handle,
            &SimpleTextOutputProtocol::GUID,
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
            &SimpleTextOutputProtocol::GUID,
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
        state.system_table = UnsafeCell::new(SystemTable {
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

            runtime_services: addr_of_mut!(state.runtime_services),
            boot_services: addr_of_mut!(state.boot_services),

            number_of_configuration_table_entries: 0,
            configuration_table: ptr::null_mut(),
        });
        state.system_table.get()
    });

    entry(image, system_table)
}
