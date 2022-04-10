use crate::uefi_stub::boot::create_event;
use crate::uefi_stub::{install_owned_protocol, SharedAnyBox};
use core::ptr::{self, addr_of_mut};
use uefi::proto::console::pointer::{Pointer, PointerMode, PointerState};
use uefi::table::boot::{EventType, Tpl};
use uefi::{Event, Identify, Result, Status};
use uefi_raw::Handle;

extern "efiapi" fn reset(this: &mut Pointer, ext_verif: bool) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn get_state(this: &Pointer, state: *mut PointerState) -> Status {
    // TODO
    unsafe {
        addr_of_mut!((*state).relative_movement).write([0; 3]);
        addr_of_mut!((*state).button).write([false; 2]);
    }
    Status::SUCCESS
}

pub fn install_pointer_protocol() -> Result<Handle> {
    let mut data = SharedAnyBox::new(PointerMode {
        resolution: [1024, 1024, 0],
        has_button: [true, true],
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
    let mut interface = SharedAnyBox::new(Pointer {
        reset,
        get_state,
        wait_for_input: wait_for_input.unwrap(),
        mode: data.as_mut_ptr().cast(),
    });

    install_owned_protocol(
        None,
        Pointer::GUID,
        interface.as_mut_ptr().cast(),
        interface,
        Some(data),
    )
}
