#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![allow(stable_features)]

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::ToString;
use serde_json::json;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::table::boot::SearchType;
use uefi::table::runtime::ResetType;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let boot_services = system_table.boot_services();
    let handles = boot_services
        .locate_handle_buffer(SearchType::AllHandles)
        .unwrap();

    let mut all_protocols = BTreeSet::new();
    for handle in handles.handles() {
        let protocols = boot_services.protocols_per_handle(*handle).unwrap();
        all_protocols
            .extend(protocols.protocols().iter().map(|guid| guid.to_string()));
    }

    let data = json!({ "protocols": all_protocols });

    let mut sfs = boot_services.get_image_file_system(image_handle).unwrap();
    let mut root = sfs.open_volume().unwrap();
    let mut file = root
        .open(
            cstr16!("sysinfo.json"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap()
        .into_regular_file()
        .unwrap();
    file.write(&serde_json::to_vec_pretty(&data).unwrap())
        .unwrap();
    file.flush().unwrap();
    root.flush().unwrap();

    // Some firmware doesn't implement flush well, so give it some extra time.
    boot_services.stall(5_000_000);

    system_table.runtime_services().reset(
        ResetType::Shutdown,
        Status::SUCCESS,
        None,
    );
}
