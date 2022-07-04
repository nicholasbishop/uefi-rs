// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
#![feature(abi_efiapi)]
// ANCHOR_END: features

use log::info;
use uefi::prelude::*;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    info!("Hello world!");
    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}
// ANCHOR_END: all
