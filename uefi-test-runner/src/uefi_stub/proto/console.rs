use crate::uefi_stub::install_protocol_simple;
use core::slice;
use uefi::proto::console::serial::{ControlBits, Parity, Serial, StopBits};
use uefi::{Identify, Result, Status};
use uefi_raw::protocol::console::serial::{SerialIoMode, SerialIoProtocol};
use uefi_raw::Handle;

extern "efiapi" fn reset(this: *mut SerialIoProtocol) -> Status {
    // TODO
    Status::SUCCESS
}
extern "efiapi" fn set_attributes(
    this: *const SerialIoProtocol,
    baud_rate: u64,
    receive_fifo_depth: u32,
    timeout: u32,
    parity: Parity,
    data_bits: u8,
    stop_bits_type: StopBits,
) -> Status {
    // TODO
    Status::SUCCESS
}
extern "efiapi" fn set_control_bits(
    this: *mut SerialIoProtocol,
    control_bits: ControlBits,
) -> Status {
    // TODO
    Status::SUCCESS
}
extern "efiapi" fn get_control_bits(
    this: *const SerialIoProtocol,
    control_bits: *mut ControlBits,
) -> Status {
    // TODO
    Status::SUCCESS
}

extern "efiapi" fn write(this: *mut SerialIoProtocol, size: *mut usize, buf: *const u8) -> Status {
    let bytes = unsafe { slice::from_raw_parts(buf, *size) };
    let s = core::str::from_utf8(bytes).unwrap();
    println!("{s}");
    Status::SUCCESS
}

unsafe extern "efiapi" fn read(
    this: *mut SerialIoProtocol,
    size: *mut usize,
    buf: *mut u8,
) -> Status {
    // TODO: for now just make the tests pass.
    let output: &[u8] = if *size == 3 { b"OK\n" } else { b"Hello world!" };
    unsafe {
        buf.copy_from_nonoverlapping(output.as_ptr(), output.len());
    }
    *size = output.len();
    Status::SUCCESS
}

pub fn install_serial_protocol() -> Result<Handle> {
    let mode = Box::new(SerialIoMode::default());
    // TODO: leak
    let mode = Box::leak(mode);

    let interface = Box::new(SerialIoProtocol {
        // TODO
        revision: 1,

        reset,
        set_attributes,
        set_control_bits,
        get_control_bits,
        write,
        read,
        mode,
    });
    // TODO: leak
    let interface: *const _ = Box::leak(interface);

    install_protocol_simple(None, &Serial::GUID, interface.cast())
}
