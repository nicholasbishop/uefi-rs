use super::fs::FsDb;
use crate::try_status;
use log::debug;
use std::alloc::{self, Layout};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::ptr::NonNull;
use std::rc::Rc;
use uefi::data_types::Align;
use uefi::proto::device_path::{DevicePath, FfiDevicePath};
use uefi::table::boot::{
    EventType, InterfaceType, MemoryAttribute, MemoryDescriptor, MemoryMapKey, MemoryType,
    ProtocolSearchKey, Tpl,
};
use uefi::{Char16, Event, Guid, Handle, Identify, Result, Status};

enum ProtocolInterface {
    Owned {
        interface: SharedAnyBox,
        data: Option<SharedAnyBox>,
    },
    Raw(*mut c_void),
}

impl ProtocolInterface {
    fn as_mut_ptr(&mut self) -> *mut c_void {
        match self {
            Self::Owned { interface, .. } => interface.as_mut_ptr().cast(),
            Self::Raw(ptr) => *ptr,
        }
    }
}

struct ProtocolWrapper {
    interface: ProtocolInterface,
    in_use: bool,
}

type HandleImpl = HashMap<Guid, ProtocolWrapper>;

struct EventImpl {
    ty: EventType,
    notify_func: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
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

pub struct SharedAnyBox {
    ptr: *mut dyn Any,
    layout: Layout,
}

impl SharedAnyBox {
    pub fn new<T: 'static>(val: T) -> Self {
        let layout = Layout::for_value(&val);
        let ptr = unsafe {
            let ptr: *mut T = alloc::alloc(layout).cast();
            ptr.write(val);
            ptr
        };
        Self { ptr, layout }
    }

    pub fn as_mut_ptr(&mut self) -> *mut dyn Any {
        self.ptr
    }
}

impl Drop for SharedAnyBox {
    fn drop(&mut self) {
        // TODO: is this right? should test
        unsafe {
            ptr::drop_in_place(self.ptr);
            alloc::dealloc(self.ptr.cast(), self.layout);
        }
    }
}

pub struct SharedBox<T: ?Sized> {
    ptr: *mut T,
    layout: Layout,
}

impl<T: ?Sized + ptr_meta::Pointee> SharedBox<T> {
    pub fn new(val: &T) -> Self {
        let layout = Layout::for_value(val);
        let ptr = unsafe {
            let alloc_ptr: *mut u8 = alloc::alloc(layout).cast();
            let out_ptr: *mut T =
                ptr_meta::from_raw_parts_mut(alloc_ptr.cast(), ptr_meta::metadata(val));
            // TODO: pretty sure this is wrong since `val` could have
            // uninitialized padding bytes, but I'm not sure what the right way
            // to do it is.
            let src_ptr: *const T = val;
            alloc_ptr.copy_from_nonoverlapping(src_ptr.cast(), mem::size_of_val(val));
            out_ptr
        };
        Self { ptr, layout }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl<T: ?Sized> Drop for SharedBox<T> {
    fn drop(&mut self) {
        // TODO: is this right? should test
        unsafe {
            ptr::drop_in_place(self.ptr);
            alloc::dealloc(self.ptr.cast(), self.layout);
        }
    }
}

pub struct State {
    handle_db: HashMap<Handle, Box<HandleImpl>>,
    events: HashMap<Event, Box<EventImpl>>,
    /// TODO: not sure what the right interface here is yet.
    ///
    /// The idea is that this is a generic object store. Example: each protocol
    /// can allocate itself into this array as a Box<Any>. If the protocol
    /// itself has an interior pointer to some additional data, it can be stored
    /// here as well. This allows for proper clean up of protocols we allocate,
    /// which install_protocol_interface doesn't really support.
    #[allow(dead_code)]
    objects: Vec<SharedAnyBox>,
    pages: Vec<Pages>,
    memory_descriptors: Vec<MemoryDescriptor>,
    pub fs_db: FsDb,
}

// All "global" state goes in this thread local block. UEFI is single
// threaded, so we have types like `Protocols` that can't be shared
// between threads.
thread_local! {
    pub static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State {
        handle_db: HashMap::new(),
        events: HashMap::new(),
        objects: Vec::new(),
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
    }));
}

pub fn store_object<T: 'static>(object: T) -> *mut T {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let mut object = SharedAnyBox::new(object);
        let ptr: *mut dyn Any = object.as_mut_ptr();
        state.objects.push(object);
        ptr.cast()
    })
}

// TODO: copied from boot.rs
type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: Option<NonNull<c_void>>);

pub unsafe extern "efiapi" fn raise_tpl(new_tpl: Tpl) -> Tpl {
    todo!()
}

pub unsafe extern "efiapi" fn restore_tpl(old_tpl: Tpl) {
    todo!()
}

pub extern "efiapi" fn allocate_pages(
    alloc_ty: u32,
    mem_ty: MemoryType,
    count: usize,
    addr: &mut *mut u8,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let pages = &mut state.pages;

        let mut new_pages = Pages::new(count);
        *addr = new_pages.as_mut_ptr();
        pages.push(new_pages);

        Status::SUCCESS
    })
}

pub extern "efiapi" fn free_pages(addr: u64, pages: usize) -> Status {
    // TODO: for now just let pages leak.
    Status::SUCCESS
}

pub unsafe extern "efiapi" fn get_memory_map(
    size: &mut usize,
    map: *mut MemoryDescriptor,
    key: &mut MemoryMapKey,
    desc_size: &mut usize,
    desc_version: &mut u32,
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

pub extern "efiapi" fn allocate_pool(
    pool_type: MemoryType,
    size: usize,
    buffer: &mut *mut u8,
) -> Status {
    let num_pages = PageAlignment::round_up_to_alignment(size);

    let mut addr = ptr::null_mut();
    try_status!(allocate_pages(0, pool_type, num_pages, &mut addr));
    *buffer = addr as *mut u8;

    Status::SUCCESS
}

pub extern "efiapi" fn free_pool(buffer: *mut u8) -> Status {
    // TODO
    Status::SUCCESS
}

pub unsafe extern "efiapi" fn create_event(
    ty: EventType,
    notify_tpl: Tpl,
    notify_func: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
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
        let event = Event(NonNull::new(event_impl_ptr.cast()).unwrap());

        state.events.insert(event.unsafe_clone(), event_impl);
        *out_event = event;

        Status::SUCCESS
    })
}

pub unsafe extern "efiapi" fn set_timer(event: Event, ty: u32, trigger_time: u64) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

pub unsafe extern "efiapi" fn wait_for_event(
    number_of_events: usize,
    events: *mut Event,
    out_index: *mut usize,
) -> Status {
    *out_index = 0;
    // TODO: for now, just pretend.
    Status::SUCCESS
}

pub extern "efiapi" fn signal_event(event: Event) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn close_event(event: Event) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn check_event(event: Event) -> Status {
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

pub fn install_protocol(
    mut handle: Option<Handle>,
    guid: Guid,
    interface: *mut c_void,
) -> Result<Handle> {
    // TODO: just make install_protocol unsafe, or remove entirely
    unsafe {
        install_protocol_interface(
            &mut handle,
            &guid,
            InterfaceType::NATIVE_INTERFACE,
            interface,
        )
        .into_with_val(|| handle.unwrap())
    }
}

// TODO: dedup
pub fn install_owned_protocol(
    handle: Option<Handle>,
    guid: Guid,
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
            let handle = unsafe { Handle::from_ptr(handle_impl_ptr.cast()) }.unwrap();

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
                interface: ProtocolInterface::Owned { interface, data },
                in_use: false,
            },
        );

        Ok(handle)
    })
}

pub unsafe extern "efiapi" fn install_protocol_interface(
    in_out_handle: &mut Option<Handle>,
    guid: &Guid,
    interface_type: InterfaceType,
    interface: *mut c_void,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let db = &mut state.handle_db;

        let handle = if let Some(handle) = in_out_handle {
            *handle
        } else {
            // Create a new handle.

            let mut handle_impl = Box::new(HandleImpl::default());
            let handle_impl_ptr = handle_impl.as_mut() as *mut HandleImpl;
            let handle = unsafe { Handle::from_ptr(handle_impl_ptr.cast()) }.unwrap();

            db.insert(handle, handle_impl);
            *in_out_handle = Some(handle);
            handle
        };

        let group = db.get_mut(&handle).unwrap();

        // Not allowed to have Duplicate protocols on a handle.
        if group.contains_key(&guid) {
            return Status::INVALID_PARAMETER;
        }

        group.insert(
            *guid,
            ProtocolWrapper {
                interface: ProtocolInterface::Raw(interface),
                in_use: false,
            },
        );

        Status::SUCCESS
    })
}

pub unsafe extern "efiapi" fn reinstall_protocol_interface(
    handle: Handle,
    protocol: &Guid,
    old_interface: *mut c_void,
    new_interface: *mut c_void,
) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

pub unsafe extern "efiapi" fn uninstall_protocol_interface(
    handle: Handle,
    protocol: &Guid,
    interface: *mut c_void,
) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

pub extern "efiapi" fn handle_protocol(
    handle: Handle,
    proto: &Guid,
    out_proto: &mut *mut c_void,
) -> Status {
    todo!()
}

pub extern "efiapi" fn register_protocol_notify(
    protocol: &Guid,
    event: Event,
    registration: *mut ProtocolSearchKey,
) -> Status {
    STATE.with(|state| {
        let state = state.borrow();

        // TODO
        let search_key: *mut _ = Box::leak(Box::new(()));
        unsafe {
            *registration = ProtocolSearchKey(NonNull::new(search_key.cast::<c_void>()).unwrap());
        }

        // TODO: for now, just pretend.
        Status::SUCCESS
    })
}

fn find_handles_impl(
    search_ty: i32,
    proto: Option<&Guid>,
    key: Option<ProtocolSearchKey>,
) -> Vec<Handle> {
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
                    if v.contains_key(proto.unwrap()) {
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

pub unsafe extern "efiapi" fn locate_handle(
    search_ty: i32,
    proto: Option<&Guid>,
    key: Option<ProtocolSearchKey>,
    buf_sz: &mut usize,
    buf: *mut MaybeUninit<Handle>,
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

pub unsafe extern "efiapi" fn locate_device_path(
    proto: &Guid,
    device_path: &mut *const FfiDevicePath,
    out_handle: &mut MaybeUninit<Handle>,
) -> Status {
    // Very TODO: for now just grab the first handle we find with a
    // `DevicePath` protocol.
    STATE.with(|state| {
        let state = state.borrow();

        for (handle, pg) in state.handle_db.iter() {
            if pg.contains_key(&DevicePath::GUID) {
                if pg.contains_key(proto) {
                    out_handle.write(*handle);
                    return Status::SUCCESS;
                }
            }
        }

        Status::NOT_FOUND
    })
}

pub unsafe extern "efiapi" fn load_image(
    boot_policy: u8,
    parent_image_handle: Handle,
    device_path: *const FfiDevicePath,
    source_buffer: *const u8,
    source_size: usize,
    image_handle: &mut MaybeUninit<Handle>,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn start_image(
    image_handle: Handle,
    exit_data_size: *mut usize,
    exit_data: &mut *mut Char16,
) -> Status {
    todo!()
}

pub extern "efiapi" fn exit(
    image_handle: Handle,
    exit_status: Status,
    exit_data_size: usize,
    exit_data: *mut Char16,
) -> ! {
    todo!()
}

pub extern "efiapi" fn unload_image(image_handle: Handle) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn exit_boot_services(
    image_handle: Handle,
    map_key: MemoryMapKey,
) -> Status {
    todo!()
}

pub extern "efiapi" fn stall(microseconds: usize) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn set_watchdog_timer(
    timeout: usize,
    watchdog_code: u64,
    data_size: usize,
    watchdog_data: *const u16,
) -> Status {
    // TODO: for now, just pretend.
    Status::SUCCESS
}

pub unsafe extern "efiapi" fn connect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    remaining_device_path: *const FfiDevicePath,
    recursive: bool,
) -> Status {
    // TODO
    Status::SUCCESS
}

pub unsafe extern "efiapi" fn disconnect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    child: Option<Handle>,
) -> Status {
    // TODO
    Status::SUCCESS
}

pub extern "efiapi" fn open_protocol(
    handle: Handle,
    protocol: &Guid,
    interface: &mut *mut c_void,
    agent_handle: Handle,
    controller_handle: Option<Handle>,
    attributes: u32,
) -> Status {
    debug!("opening protocol {protocol} for handle {handle:?}");

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        if let Some(pg) = state.handle_db.get_mut(&handle) {
            if let Some(pw) = pg.get_mut(protocol) {
                // TODO: only matters for exclusive access
                assert!(!pw.in_use);

                pw.in_use = true;
                *interface = pw.interface.as_mut_ptr();

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

pub extern "efiapi" fn close_protocol(
    handle: Handle,
    protocol: &Guid,
    agent_handle: Handle,
    controller_handle: Option<Handle>,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        if let Some(pg) = state.handle_db.get_mut(&handle) {
            if let Some(pw) = pg.get_mut(protocol) {
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

pub unsafe extern "efiapi" fn protocols_per_handle(
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

pub unsafe extern "efiapi" fn locate_handle_buffer(
    search_ty: i32,
    proto: Option<&Guid>,
    key: Option<ProtocolSearchKey>,
    no_handles: &mut usize,
    buf: &mut *mut Handle,
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

pub extern "efiapi" fn locate_protocol(
    protocol_guid: &Guid,
    registration: *mut c_void,
    out_proto: &mut *mut c_void,
) -> Status {
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        // Look for any handle that implements the protocol.
        for handle_impl in state.handle_db.values_mut() {
            if let Some(pw) = handle_impl.get_mut(protocol_guid) {
                *out_proto = pw.interface.as_mut_ptr();
                return Status::SUCCESS;
            }
        }

        Status::NOT_FOUND
    })
}

pub unsafe extern "efiapi" fn copy_mem(dest: *mut u8, src: *const u8, len: usize) {
    for i in 0..len {
        dest.add(i).write(src.add(i).read());
    }
}

pub unsafe extern "efiapi" fn set_mem(buffer: *mut u8, len: usize, value: u8) {
    for i in 0..len {
        buffer.add(i).write(value);
    }
}

pub unsafe extern "efiapi" fn create_event_ex(
    ty: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
    event_group: Option<NonNull<Guid>>,
    out_event: *mut Event,
) -> Status {
    todo!()
}
