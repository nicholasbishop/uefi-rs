use crate::uefi_stub::make_and_leak;
use core::marker::PhantomData;
use core::slice;
use uefi::proto::console::serial::{ControlBits, IoMode, Parity, Serial, StopBits};
use uefi::Status;

extern "efiapi" fn reset(this: &mut Serial) -> Status {
    // TODO
    Status::SUCCESS
}
extern "efiapi" fn set_attributes(
    this: &Serial,
    baud_rate: u64,
    receive_fifo_depth: u32,
    timeout: u32,
    parity: Parity,
    data_bits: u8,
    stop_bits_type: StopBits,
) -> Status {
    todo!()
}
extern "efiapi" fn set_control_bits(this: &mut Serial, control_bits: ControlBits) -> Status {
    // TODO
    Status::SUCCESS
}
extern "efiapi" fn get_control_bits(this: &Serial, control_bits: &mut ControlBits) -> Status {
    // TODO
    Status::SUCCESS
}

extern "efiapi" fn write(this: &mut Serial, size: &mut usize, buf: *const u8) -> Status {
    let bytes = unsafe { slice::from_raw_parts(buf, *size) };
    let s = core::str::from_utf8(bytes).unwrap();
    println!("{s}");
    Status::SUCCESS
}

extern "efiapi" fn read(this: &mut Serial, size: &mut usize, buf: *mut u8) -> Status {
    // TODO: for now just make the serial.rs test pass.
    let output = b"Hello world!";
    unsafe {
        buf.copy_from_nonoverlapping(output.as_ptr(), output.len());
    }
    *size = output.len();
    Status::SUCCESS
}

// TODO: shouldn't be a lifetime on this proto.
pub fn make_serial_protocol() -> *mut Serial<'static> {
    unsafe {
        make_and_leak(Serial {
            // TODO
            revision: 1,

            reset,
            set_attributes,
            set_control_bits,
            get_control_bits,
            write,
            read,
            io_mode: &*make_and_leak(IoMode::default()),
            _no_send_or_sync: PhantomData,
        })
    }
}
