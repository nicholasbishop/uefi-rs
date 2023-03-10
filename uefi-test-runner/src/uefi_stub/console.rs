use crate::uefi_stub::make_and_leak;
use core::marker::PhantomData;
use uefi::proto::console::serial::{ControlBits, IoMode, Parity, Serial, StopBits};
use uefi::Status;

pub extern "efiapi" fn reset(this: &mut Serial) -> Status {
    todo!()
}
pub extern "efiapi" fn set_attributes(
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
pub extern "efiapi" fn set_control_bits(this: &mut Serial, control_bits: ControlBits) -> Status {
    todo!()
}
pub extern "efiapi" fn get_control_bits(this: &Serial, control_bits: &mut ControlBits) -> Status {
    todo!()
}
pub extern "efiapi" fn write(this: &mut Serial, size: &mut usize, buf: *const u8) -> Status {
    todo!()
}
pub extern "efiapi" fn read(this: &mut Serial, size: &mut usize, buf: *mut u8) -> Status {
    todo!()
}

// TODO: shouldn't be a lifetime on this proto.
fn make_serial_protocol() -> *mut Serial<'static> {
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
