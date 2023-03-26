//! `DevicePathToText` and `DevicePathFromText` Protocol

// Note on return types: the specification of the conversion functions
// is a little unusual in that they return a pointer rather than
// `EFI_STATUS`. A NULL pointer is used to indicate an error, and the
// spec says that will only happen if the input pointer is null (which
// can't happen here since we use references as input, not pointers), or
// if there is insufficient memory. So we treat any NULL output as an
// `OUT_OF_RESOURCES` error.

use crate::{proto::device_path::FfiDevicePath, proto::unsafe_protocol, Char16};

/// This struct is a wrapper of `display_only` parameter
/// used by Device Path to Text protocol.
///
/// The `display_only` parameter controls whether the longer
/// (parseable)  or shorter (display-only) form of the conversion
/// is used. If `display_only` is TRUE, then the shorter text
/// representation of the display node is used, where applicable.
/// If `display_only` is FALSE, then the longer text representation
/// of the display node is used.
#[derive(Clone, Copy, Debug)]
pub struct DisplayOnly(pub bool);

/// This struct is a wrapper of `allow_shortcuts` parameter
/// used by Device Path to Text protocol.
///
/// The `allow_shortcuts` is FALSE, then the shortcut forms of
/// text representation for a device node cannot be used. A
/// shortcut form is one which uses information other than the
/// type or subtype. If `allow_shortcuts is TRUE, then the
/// shortcut forms of text representation for a device node
/// can be used, where applicable.
#[derive(Clone, Copy, Debug)]
pub struct AllowShortcuts(pub bool);

/// Device Path to Text protocol.
///
/// This protocol provides common utility functions for converting device
/// nodes and device paths to a text representation.
#[repr(C)]
#[unsafe_protocol("8b843e20-8132-4852-90cc-551a4e4a7f1c")]
pub struct DevicePathToText {
    pub convert_device_node_to_text: unsafe extern "efiapi" fn(
        device_node: *const FfiDevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
    pub convert_device_path_to_text: unsafe extern "efiapi" fn(
        device_path: *const FfiDevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
}

/// Device Path from Text protocol.
///
/// This protocol provides common utilities for converting text to
/// device paths and device nodes.
#[repr(C)]
#[unsafe_protocol("05c99a21-c70f-4ad2-8a5f-35df3343f51e")]
pub struct DevicePathFromText {
    pub convert_text_to_device_node:
        unsafe extern "efiapi" fn(text_device_node: *const Char16) -> *const FfiDevicePath,
    pub convert_text_to_device_path:
        unsafe extern "efiapi" fn(text_device_path: *const Char16) -> *const FfiDevicePath,
}
