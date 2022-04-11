use log::debug;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{self, MaybeUninit};
use std::ptr::NonNull;
use std::rc::Rc;
use uefi::proto::device_path::DevicePath;
use uefi::table::boot::{
    EventType, MemoryAttribute, MemoryDescriptor, MemoryMapKey, MemoryType, Tpl,
};
use uefi::{Char16, Error, Event, Guid, Handle, Identify, Result, Status};

struct ProtocolWrapper {
    protocol: Box<dyn Any>,
    in_use: bool,
}

type ProtocolGroup = HashMap<Guid, ProtocolWrapper>;

pub struct HandleDb {
    handles: HashMap<Handle, ProtocolGroup>,
    next_handle_val: usize,
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

    fn physical_address(&self) -> u64 {
        let addr = self.data.as_ptr() as u64;
        // Round up to a page boundary. We allocate enough extra space
        // to make this OK.
        let r = addr % 4096;
        if r == 0 {
            addr
        } else {
            addr + (4096 - r)
        }
    }
}

#[derive(Default)]
pub struct MemoryMap {
    descriptors: Vec<MemoryDescriptor>,
}

thread_local! {
    pub static HANDLE_DB: Rc<RefCell<HandleDb>> = Rc::new(RefCell::new(HandleDb {
        handles: HashMap::new(),
        next_handle_val: 1,
    }));

    pub static PAGES: Rc<RefCell<Vec<Pages>>> = Rc::default();

    pub static MEM_MAP: Rc<RefCell<MemoryMap>> = Rc::new(RefCell::new(MemoryMap {
        // TODO
        descriptors: vec![MemoryDescriptor {
            ty: MemoryType::LOADER_CODE,
            padding: 0,
            phys_start: 0,
            virt_start: 0,
            page_count: 1,
            att: MemoryAttribute::empty(),
        }],
    }));
}

pub fn install_protocol(
    handle: Option<Handle>,
    guid: Guid,
    interface: Box<dyn Any>,
) -> Result<Handle> {
    HANDLE_DB.with(|db| {
        let mut db = db.borrow_mut();

        let handle = if let Some(handle) = handle {
            handle
        } else {
            // Create a new handle.

            let val = db.next_handle_val;
            db.next_handle_val += 1;

            let handle = unsafe { Handle::from_ptr(val as *mut _).unwrap() };
            db.handles.insert(handle, ProtocolGroup::default());
            handle
        };

        let group = db.handles.get_mut(&handle).unwrap();

        // Not allowed to have Duplicate protocols on a handle.
        if group.contains_key(&guid) {
            return Err(Error::from(Status::INVALID_PARAMETER));
        }

        group.insert(
            guid,
            ProtocolWrapper {
                protocol: interface,
                in_use: false,
            },
        );

        Status::SUCCESS.into_with_val(|| handle)
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
    addr: &mut u64,
) -> Status {
    PAGES.with(|pages| {
        let mut pages = pages.borrow_mut();

        let new_pages = Pages::new(count);
        *addr = new_pages.physical_address();
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
    MEM_MAP.with(|mem_map| {
        let mem_map = mem_map.borrow();

        let current_size = *size;
        let required_size = mem_map.descriptors.len() * mem::size_of::<MemoryDescriptor>();

        // Set output sizes.
        *size = required_size;
        *desc_size = mem::size_of::<MemoryDescriptor>();

        if current_size < required_size {
            return Status::BUFFER_TOO_SMALL;
        }

        for (i, desc) in mem_map.descriptors.iter().enumerate() {
            map.add(i).write(*desc);
        }
        Status::SUCCESS
    })
}

pub extern "efiapi" fn allocate_pool(
    pool_type: MemoryType,
    size: usize,
    buffer: &mut *mut u8,
) -> Status {
    todo!()
}

pub extern "efiapi" fn free_pool(buffer: *mut u8) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn create_event(
    ty: EventType,
    notify_tpl: Tpl,
    notify_func: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
    out_event: *mut Event,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn set_timer(event: Event, ty: u32, trigger_time: u64) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn wait_for_event(
    number_of_events: usize,
    events: *mut Event,
    out_index: *mut usize,
) -> Status {
    todo!()
}

pub extern "efiapi" fn signal_event(event: Event) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn close_event(event: Event) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn check_event(event: Event) -> Status {
    todo!()
}

pub extern "efiapi" fn handle_protocol(
    handle: Handle,
    proto: &Guid,
    out_proto: &mut *mut c_void,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn locate_handle(
    search_ty: i32,
    proto: *const Guid,
    key: *mut c_void,
    buf_sz: &mut usize,
    buf: *mut MaybeUninit<Handle>,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn locate_device_path(
    proto: &Guid,
    device_path: &mut &DevicePath,
    out_handle: &mut MaybeUninit<Handle>,
) -> Status {
    // Very TODO: for now just grab the first handle we find with a
    // `DevicePath` protocol.
    HANDLE_DB.with(|db| {
        let db = db.borrow();

        for (handle, pg) in db.handles.iter() {
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
    device_path: *const DevicePath,
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
    todo!()
}

pub unsafe extern "efiapi" fn connect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    remaining_device_path: *const DevicePath,
    recursive: bool,
) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn disconnect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    child: Option<Handle>,
) -> Status {
    todo!()
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

    HANDLE_DB.with(|db| {
        let mut db = db.borrow_mut();

        if let Some(pg) = db.handles.get_mut(&handle) {
            if let Some(pw) = pg.get_mut(protocol) {
                // TODO: only matters for exclusive access
                assert!(!pw.in_use);

                pw.in_use = true;
                *interface = pw.protocol.as_mut() as *mut _ as *mut c_void;

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
    HANDLE_DB.with(|db| {
        let mut db = db.borrow_mut();

        if let Some(pg) = db.handles.get_mut(&handle) {
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
    todo!()
}

pub unsafe extern "efiapi" fn locate_handle_buffer(
    search_ty: i32,
    proto: *const Guid,
    key: *const c_void,
    no_handles: &mut usize,
    buf: &mut *mut Handle,
) -> Status {
    todo!()
}

pub extern "efiapi" fn locate_protocol(
    proto: &Guid,
    registration: *mut c_void,
    out_proto: &mut *mut c_void,
) -> Status {
    todo!()
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
