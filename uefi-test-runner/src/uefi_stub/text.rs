use crate::uefi_stub::{install_owned_protocol, SharedAnyBox};
use std::marker::PhantomData;
use std::ptr;
use uefi::proto::console::text::{Output, OutputData};
use uefi::proto::device_path::text::{DevicePathFromText, DevicePathToText};
use uefi::proto::device_path::FfiDevicePath;
use uefi::{Char16, Handle, Identify, Result, Status};

extern "efiapi" fn reset(this: &Output, extended: bool) -> Status {
    Status::SUCCESS
}

unsafe extern "efiapi" fn output_string(this: &Output, string: *const Char16) -> Status {
    todo!()
}

unsafe extern "efiapi" fn test_string(this: &Output, string: *const Char16) -> Status {
    todo!()
}

extern "efiapi" fn query_mode(
    this: &Output,
    mode: usize,
    columns: &mut usize,
    rows: &mut usize,
) -> Status {
    *columns = 80;
    *rows = 25;
    Status::SUCCESS
}

extern "efiapi" fn set_mode(this: &mut Output, mode: usize) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn set_attribute(this: &mut Output, attribute: usize) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn clear_screen(this: &mut Output) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn set_cursor_position(this: &mut Output, column: usize, row: usize) -> Status {
    Status::SUCCESS
}

extern "efiapi" fn enable_cursor(this: &mut Output, visible: bool) -> Status {
    Status::SUCCESS
}

pub fn install_output_protocol() -> Result<Handle> {
    let mut output_data = SharedAnyBox::new(OutputData {
        max_mode: 1,
        mode: 0,
        attribute: 0,
        cursor_column: 0,
        cursor_row: 0,
        cursor_visible: false,
    });

    install_owned_protocol(
        None,
        Output::GUID,
        SharedAnyBox::new(Output {
            reset,
            output_string,
            test_string,
            query_mode,
            set_mode,
            set_attribute,
            clear_screen,
            set_cursor_position,
            enable_cursor,
            data: output_data.as_mut_ptr().cast(),
            _no_send_or_sync: PhantomData,
        }),
        Some(output_data),
    )
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

pub fn make_device_path_to_text() -> DevicePathToText {
    DevicePathToText {
        convert_device_node_to_text,
        convert_device_path_to_text,
        _no_send_or_sync: PhantomData,
    }
}

pub fn make_device_path_from_text() -> DevicePathFromText {
    DevicePathFromText {
        convert_text_to_device_node,
        convert_text_to_device_path,
        _no_send_or_sync: PhantomData,
    }
}
