use crate::uefi_stub::{install_owned_protocol, with_owned_protocol_data, SharedAnyBox};
use core::ptr::addr_of;
use core::{mem, ptr};
use uefi::proto::console::gop::{
    BltPixel, GraphicsOutput, ModeData, ModeInfo, PixelBitmask, PixelFormat,
};
use uefi::{Identify, Result, Status};
use uefi_raw::Handle;

type ProtocolData = (ModeData, ModeInfo, Vec<u8>);

extern "efiapi" fn query_mode(
    this: &GraphicsOutput,
    mode: u32,
    info_sz: &mut usize,
    info: &mut *const ModeInfo,
) -> Status {
    *info_sz = mem::size_of::<ModeInfo>();
    // TODO: assuming just one mode
    with_owned_protocol_data::<ProtocolData, _, _>(this, |data| {
        *info = &data.1;
    })
    .unwrap();

    Status::SUCCESS
}

extern "efiapi" fn set_mode(this: &mut GraphicsOutput, mode: u32) -> Status {
    // TODO
    Status::SUCCESS
}

unsafe extern "efiapi" fn blt(
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
) -> Status {
    // TODO
    Status::SUCCESS
}

pub fn install_gop_protocol() -> Result<Handle> {
    let mut data = SharedAnyBox::new((
        ModeData {
            // TODO
            max_mode: 1,
            mode: 0,
            info: ptr::null(),
            info_sz: mem::size_of::<ModeInfo>(),
            fb_address: 0,
            fb_size: 1024 * 768 * 4,
        },
        ModeInfo {
            // TODO
            version: 1,
            hor_res: 1024,
            ver_res: 768,
            format: PixelFormat::Rgb,
            mask: PixelBitmask {
                // TODO
                red: 0,
                green: 0,
                blue: 0,
                reserved: 0,
            },
            stride: 0,
        },
        // Framebuffer
        vec![0u8; 1024 * 768 * 4],
    ));
    let tmp = data.downcast_mut::<ProtocolData>().unwrap();
    tmp.0.info = &tmp.1;
    tmp.0.fb_address = tmp.2.as_ptr() as u64;
    let mut interface = SharedAnyBox::new(GraphicsOutput {
        query_mode,
        set_mode,
        blt,
        mode: addr_of!(tmp.0),
    });

    install_owned_protocol(
        None,
        GraphicsOutput::GUID,
        interface.as_mut_ptr().cast(),
        interface,
        Some(data),
    )
}
