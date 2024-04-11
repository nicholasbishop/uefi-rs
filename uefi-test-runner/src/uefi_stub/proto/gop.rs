use crate::uefi_stub::install_protocol_simple;
use core::mem;
use uefi::Result;
use uefi_raw::protocol::console::{
    GraphicsOutputBltOperation, GraphicsOutputBltPixel, GraphicsOutputModeInformation,
    GraphicsOutputProtocol, GraphicsOutputProtocolMode, GraphicsPixelFormat, PixelBitmask,
};
use uefi_raw::{Handle, Status};

unsafe extern "efiapi" fn query_mode(
    this: *const GraphicsOutputProtocol,
    mode: u32,
    info_sz: *mut usize,
    info: *mut *const GraphicsOutputModeInformation,
) -> Status {
    *info_sz = mem::size_of::<GraphicsOutputModeInformation>();
    // TODO: assuming just one mode
    // with_owned_protocol_data::<ProtocolData, _, _>(&*this, |data| {
    //     *info = &data.1;
    // })
    // .unwrap();

    Status::SUCCESS
}

extern "efiapi" fn set_mode(this: *mut GraphicsOutputProtocol, mode: u32) -> Status {
    // TODO
    Status::SUCCESS
}

unsafe extern "efiapi" fn blt(
    this: *mut GraphicsOutputProtocol,
    buffer: *mut GraphicsOutputBltPixel,
    op: GraphicsOutputBltOperation,
    source_x: usize,
    source_y: usize,
    dest_x: usize,
    dest_y: usize,
    width: usize,
    height: usize,
    stride: usize,
) -> Status {
    // TODO
    Status::SUCCESS
}

pub fn install_gop_protocol() -> Result<Handle> {
    let framebuffer = vec![0u8; 1024 * 768 * 4].into_boxed_slice();
    // TODO: leak
    let framebuffer = Box::leak(framebuffer);

    let mode_information = Box::new(GraphicsOutputModeInformation {
        // TODO
        version: 1,
        horizontal_resolution: 1024,
        vertical_resolution: 768,
        pixel_format: GraphicsPixelFormat::PIXEL_RED_GREEN_BLUE_RESERVED_8_BIT_PER_COLOR,
        pixel_information: PixelBitmask {
            // TODO
            red: 0,
            green: 0,
            blue: 0,
            reserved: 0,
        },
        pixels_per_scan_line: 0,
    });
    // TODO: leak
    let mode_information = Box::leak(mode_information);

    let mode = Box::new(GraphicsOutputProtocolMode {
        // TODO
        max_mode: 1,
        mode: 0,
        info: mode_information,
        size_of_info: mem::size_of::<GraphicsOutputModeInformation>(),
        frame_buffer_base: 0,
        frame_buffer_size: 1024 * 768 * 4,
    });
    // TODO: leak
    let mode = Box::leak(mode);

    let interface = Box::new(GraphicsOutputProtocol {
        query_mode,
        set_mode,
        blt,
        mode,
    });
    // TODO: Leak
    let interface: *const _ = Box::leak(interface);

    install_protocol_simple(None, &GraphicsOutputProtocol::GUID, interface.cast())
}
