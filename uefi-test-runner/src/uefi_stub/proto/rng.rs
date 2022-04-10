use crate::uefi_stub::install_protocol_simple;
use std::{mem, slice};
use uefi::Result;
use uefi_raw::protocol::rng::{RngAlgorithmType, RngProtocol};
use uefi_raw::Status;

pub static RNG_PROTOCOL_INTERFACE: RngProtocol = RngProtocol { get_info, get_rng };

unsafe extern "efiapi" fn get_info(
    this: *mut RngProtocol,
    algorithm_list_size: *mut usize,
    algorithm_list: *mut RngAlgorithmType,
) -> Status {
    *algorithm_list_size = 1 * mem::size_of::<RngAlgorithmType>();
    *algorithm_list = RngAlgorithmType::ALGORITHM_RAW;

    Status::SUCCESS
}

unsafe extern "efiapi" fn get_rng(
    this: *mut RngProtocol,
    algorithm: *const RngAlgorithmType,
    value_length: usize,
    value: *mut u8,
) -> Status {
    let value = slice::from_raw_parts_mut(value, value_length);

    // TODO: for now just fill with a constant.
    value.fill(1);

    Status::SUCCESS
}

pub fn install() -> Result {
    let interface: *const _ = &RNG_PROTOCOL_INTERFACE;

    install_protocol_simple(None, &RngProtocol::GUID, interface.cast())?;

    Ok(())
}
