use crate::uefi_stub::boot::install_protocol_interface;
use crate::uefi_stub::{install_owned_protocol, SharedAnyBox};
use std::ptr;
use uefi::{CStr16, Result, Status, StatusExt};
use uefi_raw::protocol::console::{
    InputKey, SimpleTextInputProtocol, SimpleTextOutputMode, SimpleTextOutputProtocol,
};
use uefi_raw::table::boot::InterfaceType;
use uefi_raw::{Char16, Handle};

unsafe extern "efiapi" fn reset_input(
    this: *mut SimpleTextInputProtocol,
    extended_verification: bool,
) -> Status {
    Status::SUCCESS
}

unsafe extern "efiapi" fn read_key_stroke(
    this: *mut SimpleTextInputProtocol,
    key: *mut InputKey,
) -> Status {
    Status::SUCCESS
}

pub fn install_input_protocol() -> Result<Handle> {
    let mut interface = SharedAnyBox::new(SimpleTextInputProtocol {
        reset: reset_input,
        read_key_stroke,
        wait_for_key: ptr::null_mut(),
    });

    install_owned_protocol(
        None,
        SimpleTextInputProtocol::GUID,
        interface.as_mut_ptr().cast(),
        interface,
        // TODO: event data
        None,
    )
}

extern "efiapi" fn reset(this: *mut SimpleTextOutputProtocol, extended: bool) -> Status {
    Status::SUCCESS
}

unsafe extern "efiapi" fn output_string(
    this: *mut SimpleTextOutputProtocol,
    string: *const Char16,
) -> Status {
    let s = CStr16::from_ptr(string.cast());
    print!("{}", s.to_string());
    Status::SUCCESS
}

unsafe extern "efiapi" fn test_string(
    this: *mut SimpleTextOutputProtocol,
    string: *const Char16,
) -> Status {
    todo!()
}

unsafe extern "efiapi" fn query_mode(
    this: *mut SimpleTextOutputProtocol,
    mode: usize,
    columns: *mut usize,
    rows: *mut usize,
) -> Status {
    *columns = 80;
    *rows = 25;
    Status::SUCCESS
}

extern "efiapi" fn set_mode(this: *mut SimpleTextOutputProtocol, mode: usize) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn set_attribute(this: *mut SimpleTextOutputProtocol, attribute: usize) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn clear_screen(this: *mut SimpleTextOutputProtocol) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn set_cursor_position(
    this: *mut SimpleTextOutputProtocol,
    column: usize,
    row: usize,
) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn enable_cursor(this: *mut SimpleTextOutputProtocol, visible: bool) -> Status {
    Status::SUCCESS
}

#[repr(C)]
struct SimpleTextOutput {
    // TODO: signature
    mode: SimpleTextOutputMode,
    interface: SimpleTextOutputProtocol,
}

pub fn install_output_protocol() -> Result<Handle> {
    let data = SimpleTextOutputMode {
        max_mode: 1,
        mode: 0,
        attribute: 0,
        cursor_column: 0,
        cursor_row: 0,
        cursor_visible: false,
    };
    let interface = SimpleTextOutputProtocol {
        reset,
        output_string,
        test_string,
        query_mode,
        set_mode,
        set_attribute,
        clear_screen,
        set_cursor_position,
        enable_cursor,
        mode: ptr::null_mut(),
    };

    // TODO
    let p2 = Box::new(SimpleTextOutput {
        mode: data,
        interface: interface,
    });
    let p2 = Box::leak(p2);
    p2.interface.mode = ptr::addr_of_mut!(p2.mode);

    // TODO

    let mut handle = ptr::null_mut();
    unsafe {
        install_protocol_interface(
            &mut handle,
            &SimpleTextOutputProtocol::GUID,
            InterfaceType::NATIVE_INTERFACE,
            ptr::addr_of_mut!(p2.interface).cast(),
        )
    }
    .to_result_with_val(|| handle)
}
