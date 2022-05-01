use std::ptr;
use uefi::proto::console::text::Output;
use uefi::proto::device_path::FfiDevicePath;
use uefi::{Char16, Status};

pub extern "efiapi" fn reset(this: &Output, extended: bool) -> Status {
    Status::SUCCESS
}

pub unsafe extern "efiapi" fn output_string(this: &Output, string: *const Char16) -> Status {
    todo!()
}

pub unsafe extern "efiapi" fn test_string(this: &Output, string: *const Char16) -> Status {
    todo!()
}

pub extern "efiapi" fn query_mode(
    this: &Output,
    mode: usize,
    columns: &mut usize,
    rows: &mut usize,
) -> Status {
    *columns = 80;
    *rows = 25;
    Status::SUCCESS
}

pub extern "efiapi" fn set_mode(this: &mut Output, mode: usize) -> Status {
    Status::SUCCESS
}

pub extern "efiapi" fn set_attribute(this: &mut Output, attribute: usize) -> Status {
    Status::SUCCESS
}

pub extern "efiapi" fn clear_screen(this: &mut Output) -> Status {
    Status::SUCCESS
}

pub extern "efiapi" fn set_cursor_position(this: &mut Output, column: usize, row: usize) -> Status {
    Status::SUCCESS
}

pub extern "efiapi" fn enable_cursor(this: &mut Output, visible: bool) -> Status {
    Status::SUCCESS
}

pub extern "efiapi" fn convert_device_node_to_text(
    device_node: *const FfiDevicePath,
    display_only: bool,
    allow_shortcuts: bool,
) -> *const Char16 {
    // TODO
    ptr::null()
}

pub extern "efiapi" fn convert_device_path_to_text(
    device_path: *const FfiDevicePath,
    display_only: bool,
    allow_shortcuts: bool,
) -> *const Char16 {
    // TODO
    ptr::null()
}

pub extern "efiapi" fn convert_text_to_device_node(
    text_device_node: *const Char16,
) -> *const FfiDevicePath {
    // TODO
    ptr::null()
}

pub extern "efiapi" fn convert_text_to_device_path(
    text_device_path: *const Char16,
) -> *const FfiDevicePath {
    // TODO
    ptr::null()
}
