//! Graphics output protocol.
//!
//! The UEFI GOP is meant to replace existing [VGA][vga] hardware interfaces.
//!
//! The GOP provides access to a hardware frame buffer and allows UEFI apps
//! to draw directly to the graphics output device.
//!
//! The advantage of the GOP over legacy VGA is that it allows multiple GPUs
//! to exist and be used on the system. There is a GOP implementation for every
//! unique GPU in the system which supports UEFI.
//!
//! [vga]: https://en.wikipedia.org/wiki/Video_Graphics_Array
//!
//! # Definitions
//!
//! All graphics operations use a coordinate system where the top-left of the screen
//! is mapped to the point (0, 0), and `y` increases going down.
//!
//! Rectangles are defined by their top-left corner, and their width and height.
//!
//! The stride is understood as the length in bytes of a scan line / row of a buffer.
//! In theory, a buffer with a width of 640 should have (640 * 4) bytes per row,
//! but in practice there might be some extra padding used for efficiency.
//!
//! Frame buffers represent the graphics card's image buffers, backing the displays.
//!
//! Blits (**bl**ock **t**ransfer) can do high-speed memory copy between
//! the frame buffer and itself, or to and from some other buffers.
//!
//! # Blitting
//!
//! On certain hardware, the frame buffer is in a opaque format,
//! or cannot be accessed by the CPU. In those cases, it is not possible
//! to draw directly to the frame buffer. You must draw to another buffer
//! with a known pixel format, and then submit a blit command to copy that buffer
//! into the back buffer.
//!
//! Blitting can also copy a rectangle from the frame buffer to
//! another rectangle in the frame buffer, or move data out of the frame buffer
//! into a CPU-visible buffer. It can also do very fast color fills.
//!
//! The source and destination rectangles must always be of the same size:
//! no stretching / squashing will be done.
//!
//! # Animations
//!
//! UEFI does not mention if double buffering is used, nor how often
//! the frame buffer gets sent to the screen, but it's safe to assume that
//! the graphics card will re-draw the buffer at around the monitor's refresh rate.
//! You will have to implement your own double buffering if you want to
//! avoid tearing with animations.

use crate::{guid, Guid, Identify, Status};
use core::fmt::Debug;

/// Provides access to the video hardware's frame buffer.
///
/// The GOP can be used to set the properties of the frame buffer,
/// and also allows the app to access the in-memory buffer.
#[repr(C)]
pub struct GraphicsOutput {
    pub query_mode: extern "efiapi" fn(
        &GraphicsOutput,
        mode: u32,
        info_sz: &mut usize,
        &mut *const ModeInfo,
    ) -> Status,
    pub set_mode: extern "efiapi" fn(&mut GraphicsOutput, mode: u32) -> Status,
    // Clippy correctly complains that this is too complicated, but we can't change the spec.
    pub blt: unsafe extern "efiapi" fn(
        this: &mut GraphicsOutput,
        buffer: *mut BltPixel,
        op: u32,
        source_x: usize,
        source_y: usize,
        dest_x: usize,
        dest_y: usize,
        width: usize,
        height: usize,
        stride: usize,
    ) -> Status,
    pub mode: *const Mode,
}

unsafe impl Identify for GraphicsOutput {
    const GUID: Guid = guid!("9042a9de-23dc-4a38-96fb-7aded080516a");
}

#[repr(C)]
pub struct Mode {
    // Number of modes which the GOP supports.
    pub max_mode: u32,
    // Current mode.
    pub mode: u32,
    // Information about the current mode.
    pub info: *const ModeInfo,
    // Size of the above structure.
    pub info_sz: usize,
    // Physical address of the frame buffer.
    pub fb_address: u64,
    // Size in bytes. Equal to (pixel size) * height * stride.
    pub fb_size: usize,
}

/// Represents the format of the pixels in a frame buffer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum PixelFormat {
    /// Each pixel is 32-bit long, with 24-bit RGB, and the last byte is reserved.
    Rgb = 0,
    /// Each pixel is 32-bit long, with 24-bit BGR, and the last byte is reserved.
    Bgr,
    /// Custom pixel format, check the associated bitmask.
    Bitmask,
    /// The graphics mode does not support drawing directly to the frame buffer.
    ///
    /// This means you will have to use the `blt` function which will
    /// convert the graphics data to the device's internal pixel format.
    BltOnly,
    // SAFETY: UEFI also defines a PixelFormatMax variant, and states that all
    //         valid enum values are guaranteed to be smaller. Since that is the
    //         case, adding a new enum variant would be a breaking change, so it
    //         is safe to model this C enum as a Rust enum.
}

/// Bitmask used to indicate which bits of a pixel represent a given color.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct PixelBitmask {
    /// The bits indicating the red channel.
    pub red: u32,
    /// The bits indicating the green channel.
    pub green: u32,
    /// The bits indicating the blue channel.
    pub blue: u32,
    /// The reserved bits, which are ignored by the video hardware.
    pub reserved: u32,
}

/// Information about a graphics output mode.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ModeInfo {
    // The only known version, associated with the current spec, is 0.
    pub version: u32,
    pub hor_res: u32,
    pub ver_res: u32,
    pub format: PixelFormat,
    pub mask: PixelBitmask,
    pub stride: u32,
}

/// Format of pixel data used for blitting.
///
/// This is a BGR 24-bit format with an 8-bit padding, to keep each pixel 32-bit in size.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct BltPixel {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub reserved: u8,
}

impl BltPixel {
    /// Create a new pixel from RGB values.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            reserved: 0,
        }
    }
}

impl From<u32> for BltPixel {
    fn from(color: u32) -> Self {
        Self {
            blue: (color & 0x00_00_FF) as u8,
            green: ((color & 0x00_FF_00) >> 8) as u8,
            red: ((color & 0xFF_00_00) >> 16) as u8,
            reserved: 0,
        }
    }
}

/// Region of the `BltBuffer` which we are operating on
///
/// Some `Blt` operations can operate on either the full `BltBuffer` or a
/// sub-rectangle of it, but require the stride to be known in the latter case.
#[derive(Clone, Copy, Debug)]
pub enum BltRegion {
    /// Operate on the full `BltBuffer`
    Full,

    /// Operate on a sub-rectangle of the `BltBuffer`
    SubRectangle {
        /// Coordinate of the rectangle in the `BltBuffer`
        coords: (usize, usize),

        /// Stride (length of each row of the `BltBuffer`) in **pixels**
        px_stride: usize,
    },
}

/// Blit operation to perform.
#[derive(Debug)]
pub enum BltOp<'buf> {
    /// Fills a rectangle of video display with a pixel color.
    VideoFill {
        /// The color to fill with.
        color: BltPixel,
        /// The X / Y coordinates of the destination rectangle.
        dest: (usize, usize),
        /// The width / height of the rectangle.
        dims: (usize, usize),
    },
    /// Reads data from the video display to the buffer.
    VideoToBltBuffer {
        /// Buffer into which to copy data.
        buffer: &'buf mut [BltPixel],
        /// Coordinates of the source rectangle, in the frame buffer.
        src: (usize, usize),
        /// Location of the destination rectangle in the user-provided buffer
        dest: BltRegion,
        /// Width / height of the rectangles.
        dims: (usize, usize),
    },
    /// Write data from the buffer to the video rectangle.
    /// Delta must be the stride (count of bytes in a row) of the buffer.
    BufferToVideo {
        /// Buffer from which to copy data.
        buffer: &'buf [BltPixel],
        /// Location of the source rectangle in the user-provided buffer.
        src: BltRegion,
        /// Coordinates of the destination rectangle, in the frame buffer.
        dest: (usize, usize),
        /// Width / height of the rectangles.
        dims: (usize, usize),
    },
    /// Copy from the source rectangle in video memory to
    /// the destination rectangle, also in video memory.
    VideoToVideo {
        /// Coordinates of the source rectangle, in the frame buffer.
        src: (usize, usize),
        /// Coordinates of the destination rectangle, also in the frame buffer.
        dest: (usize, usize),
        /// Width / height of the rectangles.
        dims: (usize, usize),
    },
}
