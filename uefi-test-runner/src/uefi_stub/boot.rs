use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use uefi::proto::device_path::DevicePath;
use uefi::table::boot::{EventType, MemoryDescriptor, MemoryMapKey, MemoryType, Tpl};
use uefi::{Char16, Event, Guid, Handle, Status};

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
    todo!()
}

pub extern "efiapi" fn free_pages(addr: u64, pages: usize) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn get_memory_map(
    size: &mut usize,
    map: *mut MemoryDescriptor,
    key: &mut MemoryMapKey,
    desc_size: &mut usize,
    desc_version: &mut u32,
) -> Status {
    todo!()
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
    todo!()
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
    todo!()
}

pub extern "efiapi" fn close_protocol(
    handle: Handle,
    protocol: &Guid,
    agent_handle: Handle,
    controller_handle: Option<Handle>,
) -> Status {
    todo!()
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
    todo!()
}

pub unsafe extern "efiapi" fn set_mem(buffer: *mut u8, len: usize, value: u8) {
    todo!()
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
