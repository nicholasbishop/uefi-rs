use crate::try_status;
use crate::uefi_stub::shared_box::SharedAnyBox;
use crate::uefi_stub::STATE;
use log::debug;
use std::collections::HashMap;
use std::ffi::c_void;
use std::{mem, ptr};
use uefi::data_types::Align;
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::LoadedImageDevicePath;
use uefi::proto::Protocol;
use uefi::{Identify, Result};
use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::loaded_image::LoadedImageProtocol;
use uefi_raw::table::boot::{
    BootServices, EventNotifyFn, EventType, InterfaceType, MemoryDescriptor, MemoryType,
    OpenProtocolInformationEntry, Tpl,
};
use uefi_raw::table::configuration::ConfigurationTable;
use uefi_raw::table::Header;
use uefi_raw::Event;
use uefi_raw::{Char16, Guid, Handle, Status};

enum ProtocolInterface {
    Owned {
        #[allow(dead_code)]
        interface: SharedAnyBox,
        #[allow(dead_code)]
        data: Option<SharedAnyBox>,
        ptr: *mut c_void,
    },
    Raw(*mut c_void),
}

impl ProtocolInterface {
    fn as_mut_ptr(&mut self) -> *mut c_void {
        match self {
            Self::Owned { ptr, .. } => *ptr,
            Self::Raw(ptr) => *ptr,
        }
    }
}

pub struct ProtocolWrapper {
    interface: ProtocolInterface,
    in_use: bool,
}

// TODO
type MemoryMapKey = usize;
type ProtocolSearchKey = *const c_void;

pub type HandleImpl = HashMap<Guid, ProtocolWrapper>;

pub struct EventImpl {
    ty: EventType,
    notify_func: Option<EventNotifyFn>,
    notify_ctx: *mut c_void,
}

pub struct Pages {
    data: Vec<u8>,
}

impl Pages {
    fn new(num_pages: usize) -> Self {
        Self {
            // TODO: we need the alloc to be aligned to 4096. For now
            // just allocate a bunch of extra space so we can be sure to
            // make that possible for a suballocation.
            data: vec![0; 4096 * (num_pages + 1)],
        }
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        let addr = self.data.as_ptr() as usize;
        // Round up to a page boundary. We allocate enough extra space
        // to make this OK.
        let r = addr % 4096;
        let offset = if r == 0 { 0 } else { 4096 - r };
        unsafe { self.data.as_mut_ptr().add(offset) }
    }
}

unsafe extern "efiapi" fn raise_tpl(new_tpl: Tpl) -> Tpl {
    todo!()
}

unsafe extern "efiapi" fn restore_tpl(old_tpl: Tpl) {
    todo!()
}

extern "efiapi" fn allocate_pages(
    alloc_ty: u32,
    mem_ty: MemoryType,
    count: usize,
    addr: *mut u64,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let pages = &mut state.pages;

        let mut new_pages = Pages::new(count);
        unsafe { *addr = new_pages.as_mut_ptr() as u64 };
        pages.push(new_pages);

        Status::SUCCESS
    })
}

extern "efiapi" fn free_pages(addr: u64, pages: usize) -> Status {
    // TODO: for now just let pages leak.
    Status::SUCCESS
}

unsafe extern "efiapi" fn get_memory_map(
    size: *mut usize,
    map: *mut MemoryDescriptor,
    key: *mut MemoryMapKey,
    desc_size: *mut usize,
    desc_version: *mut u32,
) -> Status {
    STATE.with(|state| {
        let state = state.borrow();
        let mem_desc = &state.memory_descriptors;

        let current_size = *size;
        let required_size = mem_desc.len() * mem::size_of::<MemoryDescriptor>();

        // Set output sizes.
        *size = required_size;
        *desc_size = mem::size_of::<MemoryDescriptor>();

        if current_size < required_size {
            return Status::BUFFER_TOO_SMALL;
        }

        for (i, desc) in mem_desc.iter().enumerate() {
            map.add(i).write(*desc);
        }
        Status::SUCCESS
    })
}

struct PageAlignment;
impl Align for PageAlignment {
    fn alignment() -> usize {
        4096
    }
}

// TODO: pub
pub extern "efiapi" fn allocate_pool(
    pool_type: MemoryType,
    size: usize,
    buffer: *mut *mut u8,
) -> Status {
    let num_pages = PageAlignment::round_up_to_alignment(size);

    let mut addr = 0;
    try_status!(allocate_pages(0, pool_type, num_pages, &mut addr));
    unsafe { *buffer = addr as *mut u8 };

    Status::SUCCESS
}

extern "efiapi" fn free_pool(buffer: *mut u8) -> Status {
    // TODO
    Status::SUCCESS
}

// TODO: pub
pub unsafe extern "efiapi" fn create_event(
    ty: EventType,
    notify_tpl: Tpl,
    notify_func: Option<EventNotifyFn>,
    notify_ctx: *mut c_void,
    out_event: *mut Event,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        let mut event_impl = Box::new(EventImpl {
            ty,
            notify_func,
            notify_ctx,
        });
        let event_impl_ptr = event_impl.as_mut() as *mut EventImpl;
        let event: Event = event_impl_ptr.cast();

        state.events.insert(event, event_impl);
        *out_event = event;

        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn set_timer(event: Event, ty: u32, trigger_time: u64) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

unsafe extern "efiapi" fn wait_for_event(
    number_of_events: usize,
    events: *mut Event,
    out_index: *mut usize,
) -> Status {
    *out_index = 0;
    // TODO: for now, just pretend.
    Status::SUCCESS
}

extern "efiapi" fn signal_event(event: Event) -> Status {
    todo!()
}

unsafe extern "efiapi" fn close_event(event: Event) -> Status {
    todo!()
}

unsafe extern "efiapi" fn check_event(event: Event) -> Status {
    STATE.with(|state| {
        let state = state.borrow_mut();

        let event_state = state.events.get(&event).unwrap();

        match event_state.ty {
            // TODO: add 'signaled' state to event and check here
            EventType::NOTIFY_WAIT => {
                if let Some(func) = event_state.notify_func {
                    (func)(event, event_state.notify_ctx)
                }
            }
            _ => todo!(),
        }

        // TODO
        Status::SUCCESS
    })
}

// TODO: what if we had some new kind of OwnedProtocol trait with associated
// types for the protocol interface type and data type? Then maybe could avoid
// some of this Any stuff.

#[allow(dead_code)]
pub fn with_owned_protocol_data<D: 'static, P, F>(key: &P, f: F) -> Result
where
    P: Protocol,
    F: FnOnce(&D),
{
    let key: *const _ = key;
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let db = &mut state.handle_db;

        for handle_data in db.values_mut() {
            if let Some(protocol_wrapper) = handle_data.get(&P::GUID) {
                match &protocol_wrapper.interface {
                    ProtocolInterface::Owned { ptr, data, .. } => {
                        if ptr.cast_const() == key.cast() {
                            if let Some(data) = data {
                                if let Some(data) = data.downcast::<D>() {
                                    f(data);
                                    return Ok(());
                                } else {
                                    return Err(Status::ABORTED.into());
                                }
                            } else {
                                return Err(Status::ABORTED.into());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Err(Status::NOT_FOUND.into())
    })
}

// TODO: dedup
pub fn install_owned_protocol(
    handle: Option<Handle>,
    guid: Guid,
    interface_ptr: *mut c_void,
    interface: SharedAnyBox,
    data: Option<SharedAnyBox>,
) -> Result<Handle> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let db = &mut state.handle_db;

        let handle = if let Some(handle) = handle {
            handle
        } else {
            // Create a new handle.

            let mut handle_impl = Box::new(HandleImpl::default());
            let handle_impl_ptr = handle_impl.as_mut() as *mut HandleImpl;
            let handle: Handle = handle_impl_ptr.cast();

            db.insert(handle, handle_impl);
            handle
        };

        let group = db.get_mut(&handle).unwrap();

        // Not allowed to have Duplicate protocols on a handle.
        if group.contains_key(&guid) {
            return Err(Status::INVALID_PARAMETER.into());
        }

        group.insert(
            guid,
            ProtocolWrapper {
                interface: ProtocolInterface::Owned {
                    ptr: interface_ptr,
                    interface,
                    data,
                },
                in_use: false,
            },
        );

        Ok(handle)
    })
}

// TODO: pub
pub unsafe extern "efiapi" fn install_protocol_interface(
    in_out_handle: *mut Handle,
    guid: *const Guid,
    interface_type: InterfaceType,
    interface: *const c_void,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let db = &mut state.handle_db;

        let handle = if !(*in_out_handle).is_null() {
            *in_out_handle
        } else {
            // Create a new handle.

            let mut handle_impl = Box::new(HandleImpl::default());
            let handle_impl_ptr = handle_impl.as_mut() as *mut HandleImpl;
            let handle: Handle = handle_impl_ptr.cast();

            db.insert(handle, handle_impl);
            *in_out_handle = handle;
            handle
        };

        let group = db.get_mut(&handle).unwrap();

        // Not allowed to have Duplicate protocols on a handle.
        if group.contains_key(&*guid) {
            return Status::INVALID_PARAMETER;
        }

        group.insert(
            *guid,
            ProtocolWrapper {
                interface: ProtocolInterface::Raw(interface.cast_mut()),
                in_use: false,
            },
        );

        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn reinstall_protocol_interface(
    handle: Handle,
    protocol: *const Guid,
    old_interface: *const c_void,
    new_interface: *const c_void,
) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

unsafe extern "efiapi" fn uninstall_protocol_interface(
    handle: Handle,
    protocol: *const Guid,
    interface: *const c_void,
) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

extern "efiapi" fn handle_protocol(
    handle: Handle,
    proto: *const Guid,
    out_proto: *mut *mut c_void,
) -> Status {
    todo!()
}

extern "efiapi" fn register_protocol_notify(
    protocol: *const Guid,
    event: Event,
    registration: *mut ProtocolSearchKey,
) -> Status {
    STATE.with(|state| {
        let state = state.borrow();

        // TODO
        let search_key: *mut _ = Box::leak(Box::new(()));
        unsafe {
            *registration = search_key.cast::<c_void>();
        }

        // TODO: for now, just pretend.
        Status::SUCCESS
    })
}

fn find_handles_impl(search_ty: i32, proto: *const Guid, key: ProtocolSearchKey) -> Vec<Handle> {
    STATE.with(|state| {
        let state = state.borrow();

        match search_ty {
            // AllHandles
            0 => state.handle_db.keys().cloned().collect(),
            // ByRegisterNotify
            1 => {
                todo!();
            }
            // ByProtocol
            2 => state
                .handle_db
                .iter()
                .filter_map(|(handle, v)| {
                    if v.contains_key(unsafe { &*proto }) {
                        Some(handle)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect(),
            _ => {
                panic!("invalid {search_ty}");
            }
        }
    })
}

unsafe extern "efiapi" fn locate_handle(
    search_ty: i32,
    proto: *const Guid,
    key: ProtocolSearchKey,
    buf_sz: *mut usize,
    buf: *mut Handle,
) -> Status {
    let matched_handles = find_handles_impl(search_ty, proto, key);

    let available_size_in_bytes = *buf_sz;
    let size_in_bytes = matched_handles.len() * mem::size_of::<Handle>();
    *buf_sz = size_in_bytes;

    if available_size_in_bytes < size_in_bytes {
        return Status::BUFFER_TOO_SMALL;
    }

    let buf = buf.cast::<Handle>();
    for (i, handle) in matched_handles.iter().enumerate() {
        buf.add(i).write(*handle);
    }

    Status::SUCCESS
}

unsafe extern "efiapi" fn locate_device_path(
    proto: *const Guid,
    device_path: *mut *const DevicePathProtocol,
    out_handle: *mut Handle,
) -> Status {
    // Very TODO: for now just grab the first handle we find with a
    // `DevicePath` protocol.
    STATE.with(|state| {
        let state = state.borrow();

        for (handle, pg) in state.handle_db.iter() {
            if pg.contains_key(&DevicePath::GUID) {
                if pg.contains_key(&*proto) {
                    out_handle.write(*handle);
                    return Status::SUCCESS;
                }
            }
        }

        Status::NOT_FOUND
    })
}

extern "efiapi" fn install_configuration_table(
    guid_entry: *const Guid,
    table_ptr: *const c_void,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        // TODO: check GUID unique.
        state.configuration_tables.push(ConfigurationTable {
            vendor_guid: unsafe { *guid_entry },
            vendor_table: table_ptr.cast_mut(),
        });
        let configuration_tables_ptr = state.configuration_tables.as_mut_ptr();
        state.system_table.get_mut().configuration_table = configuration_tables_ptr;
        state
            .system_table
            .get_mut()
            .number_of_configuration_table_entries += 1;

        Status::SUCCESS
    })
}

#[repr(C)]
pub struct ImageImpl {
    interface: LoadedImageProtocol,
    data: Vec<u8>,
    loaded_image_device_path_buf: [u8; 256],
    loaded_image_device_path: *mut c_void,
}

// TODO: pub
pub unsafe extern "efiapi" fn load_image(
    boot_policy: u8,
    parent_image_handle: Handle,
    device_path: *const DevicePathProtocol,
    source_buffer: *const u8,
    source_size: usize,
    image_handle: *mut Handle,
) -> Status {
    STATE.with(|state| {
        let mut image = Box::pin(ImageImpl {
            // TODO
            data: Vec::new(),
            interface: LoadedImageProtocol {
                // TODO
                revision: 1,
                parent_handle: parent_image_handle,
                system_table: ptr::null(),

                device_handle: ptr::null_mut(),
                file_path: ptr::null(),

                reserved: ptr::null(),

                load_options_size: 0,
                load_options: ptr::null_mut(),

                image_base: source_buffer.cast(),
                image_size: source_size.try_into().unwrap(),
                image_code_type: MemoryType::LOADER_CODE,
                image_data_type: MemoryType::LOADER_DATA,
                unload: None,
            },
            loaded_image_device_path_buf: [0; 256],
            loaded_image_device_path: ptr::null_mut(),
        });

        // TODO: hacky
        if !device_path.is_null() {
            let device_path = DevicePath::from_ffi_ptr(device_path.cast());
            let bytes = device_path.as_bytes();
            image.loaded_image_device_path_buf[..bytes.len()].copy_from_slice(bytes);
            image.loaded_image_device_path = image.loaded_image_device_path_buf.as_mut_ptr().cast();
        }

        // TODO: make a real path
        // let path = DevicePathBuilder::with_buf(&mut image.loaded_image_device_path_buf)
        //     .finalize()
        //     .unwrap();
        // image.loaded_image_device_path = path.as_ffi_ptr().cast_mut().cast();

        // TODO
        assert_eq!(
            install_protocol_interface(
                image_handle,
                &LoadedImageProtocol::GUID,
                InterfaceType::NATIVE_INTERFACE,
                ptr::addr_of_mut!((*image).interface).cast(),
            ),
            Status::SUCCESS
        );

        assert_eq!(
            install_protocol_interface(
                image_handle,
                &LoadedImageDevicePath::GUID,
                InterfaceType::NATIVE_INTERFACE,
                (*image).loaded_image_device_path
            ),
            Status::SUCCESS
        );

        assert_eq!(
            install_protocol_interface(
                image_handle,
                &DevicePath::GUID,
                InterfaceType::NATIVE_INTERFACE,
                (*image).loaded_image_device_path
            ),
            Status::SUCCESS
        );

        let mut state = state.borrow_mut();
        state.images.push(image);

        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn start_image(
    image_handle: Handle,
    exit_data_size: *mut usize,
    exit_data: *mut *mut Char16,
) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn exit(
    image_handle: Handle,
    exit_status: Status,
    exit_data_size: usize,
    exit_data: *mut Char16,
) -> ! {
    todo!()
}

extern "efiapi" fn unload_image(image_handle: Handle) -> Status {
    todo!()
}

unsafe extern "efiapi" fn exit_boot_services(
    image_handle: Handle,
    map_key: MemoryMapKey,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        // TODO: clear more state, including the std handles
        state.system_table.get_mut().stdin = ptr::null_mut();
        state.system_table.get_mut().stdout = ptr::null_mut();
        state.system_table.get_mut().stderr = ptr::null_mut();
        state.system_table.get_mut().boot_services = ptr::null_mut();
        // TODO: update CRC

        // Very TODO
        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn get_next_monotonic_count(count: *mut u64) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn stall(microseconds: usize) -> Status {
    todo!()
}

unsafe extern "efiapi" fn set_watchdog_timer(
    timeout: usize,
    watchdog_code: u64,
    data_size: usize,
    watchdog_data: *const u16,
) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

unsafe extern "efiapi" fn connect_controller(
    controller: Handle,
    driver_image: Handle,
    remaining_device_path: *const DevicePathProtocol,
    recursive: bool,
) -> Status {
    // TODO
    Status::SUCCESS
}

unsafe extern "efiapi" fn disconnect_controller(
    controller: Handle,
    driver_image: Handle,
    child: Handle,
) -> Status {
    // TODO
    Status::SUCCESS
}

// TODO: pub
pub extern "efiapi" fn open_protocol(
    handle: Handle,
    protocol: *const Guid,
    interface: *mut *mut c_void,
    agent_handle: Handle,
    controller_handle: Handle,
    attributes: u32,
) -> Status {
    let protocol: &Guid = unsafe { &*protocol };

    debug!("opening protocol {protocol} for handle {handle:?}");

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        if let Some(pg) = state.handle_db.get_mut(&handle) {
            if let Some(pw) = pg.get_mut(protocol) {
                // TODO: only matters for exclusive access
                assert!(!pw.in_use);

                pw.in_use = true;
                unsafe { *interface = pw.interface.as_mut_ptr() };

                Status::SUCCESS
            } else {
                // Handle does not support protocol.
                Status::UNSUPPORTED
            }
        } else {
            debug!("invalid handle: {handle:?}");
            Status::INVALID_PARAMETER
        }
    })
}

// TODO: pub
pub extern "efiapi" fn close_protocol(
    handle: Handle,
    protocol: *const Guid,
    agent_handle: Handle,
    controller_handle: Handle,
) -> Status {
    let protocol: &Guid = unsafe { &*protocol };

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        if let Some(pg) = state.handle_db.get_mut(&handle) {
            if let Some(pw) = pg.get_mut(&*protocol) {
                // TODO: only matters for exclusive access
                assert!(pw.in_use);

                pw.in_use = false;

                Status::SUCCESS
            } else {
                // Handle does not support protocol.
                Status::NOT_FOUND
            }
        } else {
            debug!("invalid handle: {handle:?}");
            Status::INVALID_PARAMETER
        }
    })
}

unsafe extern "efiapi" fn open_protocol_information(
    handle: Handle,
    protocol: *const Guid,
    entry_buffer: *mut *const OpenProtocolInformationEntry,
    entry_count: *mut usize,
) -> Status {
    Status::SUCCESS
}

unsafe extern "efiapi" fn protocols_per_handle(
    handle: Handle,
    protocol_buffer: *mut *mut *const Guid,
    protocol_buffer_count: *mut usize,
) -> Status {
    let num_protocols = if let Some(num_protocols) = STATE.with(|state| {
        let state = state.borrow();

        let handle_impl = state.handle_db.get(&handle)?;
        Some(handle_impl.len())
    }) {
        num_protocols
    } else {
        return Status::INVALID_PARAMETER;
    };

    let mut buf = ptr::null_mut();
    try_status!(allocate_pool(
        MemoryType::CONVENTIONAL,
        num_protocols * mem::size_of::<*const Guid>(),
        &mut buf,
    ));
    let buf: *mut *const Guid = buf.cast();

    STATE.with(|state| {
        let state = state.borrow();

        let handle_impl = state.handle_db.get(&handle).unwrap();
        for (i, protocol_guid) in handle_impl.keys().enumerate() {
            buf.add(i).write(protocol_guid);
        }

        *protocol_buffer = buf.cast();
        *protocol_buffer_count = num_protocols;

        Status::SUCCESS
    })
}

unsafe extern "efiapi" fn locate_handle_buffer(
    search_ty: i32,
    proto: *const Guid,
    key: ProtocolSearchKey,
    no_handles: *mut usize,
    buf: *mut *mut Handle,
) -> Status {
    let matched_handles = find_handles_impl(search_ty, proto, key);

    *no_handles = matched_handles.len();

    let mut ptr = ptr::null_mut();
    try_status!(allocate_pool(
        MemoryType::CONVENTIONAL,
        matched_handles.len() * mem::size_of::<Handle>(),
        &mut ptr,
    ));

    let ptr = ptr.cast::<Handle>();
    for (i, handle) in matched_handles.iter().enumerate() {
        ptr.add(i).write(*handle);
    }

    *buf = ptr;

    Status::SUCCESS
}

unsafe extern "efiapi" fn locate_protocol(
    proto: *const Guid,
    registration: *mut c_void,
    out_proto: *mut *mut c_void,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        // Look for any handle that implements the protocol.
        for handle_impl in state.handle_db.values_mut() {
            if let Some(pw) = handle_impl.get_mut(&*proto) {
                *out_proto = pw.interface.as_mut_ptr();
                return Status::SUCCESS;
            }
        }

        Status::NOT_FOUND
    })
}

#[allow(dead_code)]
unsafe extern "C" fn not_implemented_c_abi() -> Status {
    unimplemented!()
}

unsafe extern "efiapi" fn calculate_crc32(
    data: *const c_void,
    data_size: usize,
    crc32: *mut u32,
) -> Status {
    Status::SUCCESS
}

unsafe extern "efiapi" fn copy_mem(dest: *mut u8, src: *const u8, len: usize) {
    for i in 0..len {
        dest.add(i).write(src.add(i).read());
    }
}

unsafe extern "efiapi" fn set_mem(buffer: *mut u8, len: usize, value: u8) {
    for i in 0..len {
        buffer.add(i).write(value);
    }
}

unsafe extern "efiapi" fn create_event_ex(
    ty: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: *mut c_void,
    event_group: *mut Guid,
    out_event: *mut Event,
) -> Status {
    todo!()
}

pub fn new_boot_services() -> BootServices {
    BootServices {
        // TODO
        header: Header::default(),

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
        reserved: ptr::null_mut(),
        register_protocol_notify,
        locate_handle,
        locate_device_path,
        install_configuration_table,
        load_image,
        start_image,
        exit,
        unload_image,
        exit_boot_services,
        get_next_monotonic_count,
        stall,
        set_watchdog_timer,
        connect_controller,
        disconnect_controller,
        open_protocol,
        close_protocol,
        open_protocol_information,
        protocols_per_handle,
        locate_handle_buffer,
        locate_protocol,
        install_multiple_protocol_interfaces: 0,
        uninstall_multiple_protocol_interfaces: 0,
        calculate_crc32,
        copy_mem,
        set_mem,
        create_event_ex,
    }
}

#[allow(dead_code)]
type F = unsafe extern "C" fn(handle: *mut Handle, ...) -> Status;
