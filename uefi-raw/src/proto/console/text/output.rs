use crate::proto::unsafe_protocol;
use crate::{Char16, Status};
use core::fmt;
use core::fmt::{Debug, Formatter};

/// Interface for text-based output devices.
///
/// It implements the fmt::Write trait, so you can use it to print text with
/// standard Rust constructs like the `write!()` and `writeln!()` macros.
///
/// # Accessing `Output` protocol
///
/// The standard output and standard error output protocols can be accessed
/// using [`SystemTable::stdout`] and [`SystemTable::stderr`], respectively.
///
/// An `Output` protocol can also be accessed like any other UEFI protocol.
/// See the [`BootServices`] documentation for more details of how to open a
/// protocol.
///
/// [`SystemTable::stdout`]: crate::table::SystemTable::stdout
/// [`SystemTable::stderr`]: crate::table::SystemTable::stderr
/// [`BootServices`]: crate::table::boot::BootServices#accessing-protocols
#[repr(C)]
#[unsafe_protocol("387477c2-69c7-11d2-8e39-00a0c969723b")]
pub struct Output {
    reset: extern "efiapi" fn(this: &Output, extended: bool) -> Status,
    output_string: unsafe extern "efiapi" fn(this: &Output, string: *const Char16) -> Status,
    test_string: unsafe extern "efiapi" fn(this: &Output, string: *const Char16) -> Status,
    query_mode: extern "efiapi" fn(
        this: &Output,
        mode: usize,
        columns: &mut usize,
        rows: &mut usize,
    ) -> Status,
    set_mode: extern "efiapi" fn(this: &mut Output, mode: usize) -> Status,
    set_attribute: extern "efiapi" fn(this: &mut Output, attribute: usize) -> Status,
    clear_screen: extern "efiapi" fn(this: &mut Output) -> Status,
    set_cursor_position: extern "efiapi" fn(this: &mut Output, column: usize, row: usize) -> Status,
    enable_cursor: extern "efiapi" fn(this: &mut Output, visible: bool) -> Status,
    data: *const OutputData,
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Output")
            .field("reset (fn ptr)", &(self.reset as *const u64))
            .field(
                "output_string (fn ptr)",
                &(self.output_string as *const u64),
            )
            .field("test_string (fn ptr)", &(self.test_string as *const u64))
            .field("query_mode (fn ptr)", &(self.query_mode as *const u64))
            .field("set_mode (fn ptr)", &(self.set_mode as *const u64))
            .field(
                "set_attribute (fn ptr)",
                &(self.set_attribute as *const u64),
            )
            .field("clear_screen (fn ptr)", &(self.clear_screen as *const u64))
            .field(
                "set_cursor_position (fn ptr)",
                &(self.set_cursor_position as *const u64),
            )
            .field(
                "enable_cursor (fn ptr)",
                &(self.enable_cursor as *const u64),
            )
            .field("data", &self.data)
            .finish()
    }
}

/// The text mode (resolution) of the output device.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OutputMode {
    index: usize,
    dims: (usize, usize),
}

impl OutputMode {
    /// Returns the index of this mode.
    #[inline]
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the width in columns.
    #[inline]
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.dims.0
    }

    /// Returns the height in rows.
    #[inline]
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.dims.1
    }
}

/// Additional data of the output device.
#[derive(Debug)]
#[repr(C)]
pub struct OutputData {
    /// The number of modes supported by the device.
    max_mode: i32,
    /// The current output mode.
    /// Negative index -1 is used to notify that no valid mode is configured
    mode: i32,
    /// The current character output attribute.
    attribute: i32,
    /// The cursor’s column.
    cursor_column: i32,
    /// The cursor’s row.
    cursor_row: i32,
    /// Whether the cursor is currently visible or not.
    cursor_visible: bool,
}

/// Colors for the UEFI console.
///
/// All colors can be used as foreground colors.
/// The first 8 colors can also be used as background colors.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub enum Color {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    White,
}
