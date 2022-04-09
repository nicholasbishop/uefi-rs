use uefi::prelude::*;

pub fn init(_st: &mut SystemTable<Boot>) -> uefi::Result {
    Status::SUCCESS.to_result()
    // TODO
}
