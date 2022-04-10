use crate::uefi_stub::{install_owned_protocol, SharedAnyBox};
use core::ptr::addr_of_mut;
use core::{mem, ptr};
use uefi::{Result, Status};
use uefi_raw::protocol::console::{
    GraphicsOutputBltOperation, GraphicsOutputBltPixel, GraphicsOutputModeInformation,
    GraphicsOutputProtocol, GraphicsOutputProtocolMode, GraphicsPixelFormat, PixelBitmask,
};
use uefi_raw::Handle;

type ProtocolData = (
    GraphicsOutputProtocolMode,
    GraphicsOutputModeInformation,
    Vec<u8>,
);

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
    let mut data = SharedAnyBox::new((
        GraphicsOutputProtocolMode {
            // TODO
            max_mode: 1,
            mode: 0,
            info: ptr::null_mut(),
            size_of_info: mem::size_of::<GraphicsOutputModeInformation>(),
            frame_buffer_base: 0,
            frame_buffer_size: 1024 * 768 * 4,
        },
        GraphicsOutputModeInformation {
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
        },
        // Framebuffer
        vec![0u8; 1024 * 768 * 4],
    ));
    let tmp = data.downcast_mut::<ProtocolData>().unwrap();
    tmp.0.info = &mut tmp.1;
    tmp.0.frame_buffer_base = tmp.2.as_ptr() as u64;
    let mut interface = SharedAnyBox::new(GraphicsOutputProtocol {
        query_mode,
        set_mode,
        blt,
        mode: addr_of_mut!(tmp.0),
    });

    install_owned_protocol(
        None,
        GraphicsOutputProtocol::GUID,
        interface.as_mut_ptr().cast(),
        interface,
        Some(data),
    )
}
