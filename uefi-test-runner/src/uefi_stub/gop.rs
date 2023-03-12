use crate::uefi_stub::{install_owned_protocol, with_owned_protocol_data, SharedAnyBox};
use core::marker::PhantomData;
use core::{mem, ptr};
use uefi::proto::console::gop::{
    BltPixel, GraphicsOutput, ModeData, ModeInfo, PixelBitmask, PixelFormat,
};
use uefi::{Handle, Identify, Result, Status};

type ProtocolData = (GraphicsOutput, ModeData, ModeInfo);

extern "efiapi" fn query_mode(
    this: &GraphicsOutput,
    mode: u32,
    info_sz: &mut usize,
    info: &mut *const ModeInfo,
) -> Status {
    *info_sz = mem::size_of::<ModeInfo>();
    // TODO: assuming just one mode
    with_owned_protocol_data::<ProtocolData, _, _>(this, |data| {
        *info = &data.2;
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
        GraphicsOutput {
            query_mode,
            set_mode,
            blt,
            mode: ptr::null(),
            _no_send_or_sync: PhantomData,
        },
        ModeData {
            // TODO
            max_mode: 1,
            mode: 0,
            info: ptr::null(),
            info_sz: mem::size_of::<ModeInfo>(),
            fb_address: 0,
            fb_size: 0,
        },
        ModeInfo {
            // TODO
            version: 1,
            hor_res: 1,
            ver_res: 1,
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
    ));
    let tmp = data.downcast_mut::<ProtocolData>().unwrap();
    tmp.1.info = &tmp.2;
    tmp.0.mode = &tmp.1;
    let interface = data.as_mut_ptr();

    install_owned_protocol(None, GraphicsOutput::GUID, interface.cast(), data)
}
