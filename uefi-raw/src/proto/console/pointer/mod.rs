//! Pointer device access.

use crate::proto::unsafe_protocol;
use crate::{Event, Status};

/// Provides information about a pointer device.
#[repr(C)]
#[unsafe_protocol("31878c87-0b75-11d5-9a4f-0090273fc14d")]
pub struct Pointer {
    pub reset: extern "efiapi" fn(this: &mut Pointer, ext_verif: bool) -> Status,
    pub get_state: extern "efiapi" fn(this: &Pointer, state: *mut PointerState) -> Status,
    pub wait_for_input: Event,
    pub mode: *const PointerMode,
}

/// Information about this pointer device.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PointerMode {
    /// The pointer device's resolution on the X/Y/Z axis in counts/mm.
    /// If a value is 0, then the device does _not_ support that axis.
    pub resolution: (u64, u64, u64),
    /// Whether the devices has a left button / right button.
    pub has_button: (bool, bool),
}

/// The relative change in the pointer's state.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PointerState {
    /// The relative movement on the X/Y/Z axis.
    ///
    /// If `PointerMode` indicates an axis is not supported, it must be ignored.
    pub relative_movement: (i32, i32, i32),
    /// Whether the left / right mouse button is currently pressed.
    ///
    /// If `PointerMode` indicates a button is not supported, it must be ignored.
    pub button: (bool, bool),
}
