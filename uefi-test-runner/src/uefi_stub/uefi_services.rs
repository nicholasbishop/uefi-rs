use uefi::prelude::*;
use uefi::StatusExt;

pub fn init(st: &mut SystemTable<Boot>) -> uefi::Result {
    unsafe { super::logger::init(st) };

    Status::SUCCESS.to_result()
    // TODO
}
