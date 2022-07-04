// ANCHOR: all
#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use log::info;
use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams, SearchType};
use uefi::Identify;

// ANCHOR: loaded_image
#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table)?;
    let boot_services = system_table.boot_services();

    let loaded_image = boot_services.open_protocol::<LoadedImage>(
        OpenProtocolParams {
            handle: image_handle,
            agent: image_handle,
            controller: None,
        },
        OpenProtocolAttributes::Exclusive,
    )?;
    // ANCHOR_END: loaded_image

    // ANCHOR: device_path
    let device_path_to_text_handle = *boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
        .handles()
        .first()
        .unwrap();

    let device_path_to_text = boot_services.open_protocol::<DevicePathToText>(
        OpenProtocolParams {
            handle: device_path_to_text_handle,
            agent: image_handle,
            controller: None,
        },
        OpenProtocolAttributes::Exclusive,
    )?;
    // ANCHOR_END: device_path

    // ANCHOR: text
    let image_device_path = loaded_image.file_path().unwrap();
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(
            boot_services,
            image_device_path,
            DisplayOnly(true),
            AllowShortcuts(false),
        )
        .unwrap();
    let image_device_path_text = &*image_device_path_text;

    info!("Image path: {}", image_device_path_text);
    // ANCHOR_END: text

    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}
// ANCHOR_END: all
