use uefi::prelude::*;

pub fn test(bt: &BootServices) {
    info!("Testing Network protocols");

    http::test(bt);
    pxe::test(bt);
    snp::test(bt);
}

mod http;
mod pxe;
mod snp;
