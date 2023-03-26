//! Data type definitions
//!
//! This module defines the basic data types that are used throughout uefi-rs

use core::{ffi::c_void, ptr::NonNull};

/// Opaque handle to an UEFI entity (protocol, image...), guaranteed to be non-null.
///
/// If you need to have a nullable handle (for a custom UEFI FFI for example) use `Option<Handle>`.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Handle(pub NonNull<c_void>);

impl Handle {
    /// Creates a new [`Handle`] from a raw address. The address might
    /// come from the Multiboot2 information structure or something similar.
    ///
    /// # Example
    /// ```no_run
    /// use core::ffi::c_void;
    /// use uefi::Handle;
    ///
    /// let image_handle_addr = 0xdeadbeef as *mut c_void;
    ///
    /// let uefi_image_handle = unsafe {
    ///     Handle::from_ptr(image_handle_addr).expect("Pointer must not be null!")
    /// };
    /// ```
    ///
    /// # Safety
    /// This function is unsafe because the caller must be sure that the pointer
    /// is valid. Otherwise, further operations on the object might result in
    /// undefined behaviour, even if the methods aren't marked as unsafe.
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        // shorthand for "|ptr| Self(ptr)"
        NonNull::new(ptr).map(Self)
    }
}

/// Handle to an event structure, guaranteed to be non-null.
///
/// If you need to have a nullable event, use `Option<Event>`.
#[repr(transparent)]
#[derive(Debug)]
pub struct Event(pub NonNull<c_void>);

impl Event {
    /// Clone this `Event`
    ///
    /// # Safety
    /// When an event is closed by calling `BootServices::close_event`, that event and ALL references
    /// to it are invalidated and the underlying memory is freed by firmware. The caller must ensure
    /// that any clones of a closed `Event` are never used again.
    #[must_use]
    pub const unsafe fn unsafe_clone(&self) -> Self {
        Self(self.0)
    }
}

/// Physical memory address. This is always a 64-bit value, regardless
/// of target platform.
pub type PhysicalAddress = u64;

/// Virtual memory address. This is always a 64-bit value, regardless
/// of target platform.
pub type VirtualAddress = u64;

pub mod guid;
pub use self::guid::Guid;

pub mod chars;
pub use self::chars::{Char16, Char8};

#[macro_use]
mod enums;
