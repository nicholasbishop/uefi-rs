use crate::uefi_stub::boot::create_event;
use crate::uefi_stub::{install_owned_protocol, SharedAnyBox};
use core::ptr;
use uefi::table::boot::{EventType, Tpl};
use uefi::{Event, Result, Status};
use uefi_raw::protocol::console::{SimplePointerMode, SimplePointerProtocol, SimplePointerState};
use uefi_raw::Handle;

extern "efiapi" fn reset(this: *mut SimplePointerProtocol, ext_verif: bool) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn get_state(
    this: *mut SimplePointerProtocol,
    state: *mut SimplePointerState,
) -> Status {
    // TODO
    unsafe {
        (*state).relative_movement_x = 0;
        (*state).relative_movement_y = 0;
        (*state).relative_movement_z = 0;
        (*state).left_button = 0;
        (*state).right_button = 0;
    }
    Status::SUCCESS
}

pub fn install_pointer_protocol() -> Result<Handle> {
    let mut data = SharedAnyBox::new(SimplePointerMode {
        resolution_x: 1024,
        resolution_y: 1024,
        resolution_z: 0,
        left_button: 1,
        right_button: 1,
    });
    let mut wait_for_input = None;
    let wait_for_input_ptr: *mut Option<Event> = &mut wait_for_input;
    // TODO: not sure if these are right
    assert!(unsafe {
        create_event(
            EventType::NOTIFY_SIGNAL,
            Tpl::CALLBACK,
            None,
            ptr::null_mut(),
            wait_for_input_ptr.cast(),
        )
    }
    .is_success());
    let mut interface = SharedAnyBox::new(SimplePointerProtocol {
        reset,
        get_state,
        wait_for_input: wait_for_input.unwrap().as_ptr(),
        mode: data.as_mut_ptr().cast(),
    });

    install_owned_protocol(
        None,
        SimplePointerProtocol::GUID,
        interface.as_mut_ptr().cast(),
        interface,
        Some(data),
    )
}
