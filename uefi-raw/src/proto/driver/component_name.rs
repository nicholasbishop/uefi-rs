// This module defines the `ComponentName1` type and marks it deprecated. That
// causes warnings for uses within this module (e.g. the `impl ComponentName1`
// block), so turn off deprecated warnings. It's not yet possible to make this
// allow more fine-grained, see https://github.com/rust-lang/rust/issues/62398.
#![allow(deprecated)]

use crate::proto::unsafe_protocol;
use crate::{Handle, Status};
use core::fmt::Debug;

#[unsafe_protocol("107a772c-d5e1-11d4-9a46-0090273fc14d")]
#[repr(C)]
pub struct ComponentName1 {
    pub get_driver_name: unsafe extern "efiapi" fn(
        this: *const Self,
        language: *const u8,
        driver_name: *mut *const u16,
    ) -> Status,
    pub get_controller_name: unsafe extern "efiapi" fn(
        this: *const Self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: *const u8,
        controller_name: *mut *const u16,
    ) -> Status,
    pub supported_languages: *const u8,
}

#[unsafe_protocol("6a7a5cff-e8d9-4f70-bada-75ab3025ce14")]
#[repr(C)]
pub struct ComponentName2 {
    pub get_driver_name: unsafe extern "efiapi" fn(
        this: *const Self,
        language: *const u8,
        driver_name: *mut *const u16,
    ) -> Status,
    pub get_controller_name: unsafe extern "efiapi" fn(
        this: *const Self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: *const u8,
        controller_name: *mut *const u16,
    ) -> Status,
    pub supported_languages: *const u8,
}

/// Error returned by [`ComponentName1::supported_languages`] and
/// [`ComponentName2::supported_languages`].
#[derive(Debug, Eq, PartialEq)]
pub enum LanguageError {
    /// The supported languages list contains a non-ASCII character at the
    /// specified index.
    Ascii {
        /// Index of the invalid character.
        index: usize,
    },
}
