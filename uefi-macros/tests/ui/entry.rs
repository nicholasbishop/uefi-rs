#![no_main]
#![feature(abi_efiapi)]

use uefi::prelude::*;
use uefi_macros::entry;

mod good_entry {
    use super::*;

    #[entry]
    fn efi_main(_handle: Handle, _st: SystemTable<Boot>) -> Status {
        Status::SUCCESS
    }
}

mod bad_attr_arg {
    use super::*;

    #[entry(some_arg)]
    extern "C" fn efi_main(_handle: Handle, _st: SystemTable<Boot>) -> Status {
        Status::SUCCESS
    }
}

mod bad_abi_modifier {
    use super::*;

    #[entry]
    extern "C" fn efi_main(_handle: Handle, _st: SystemTable<Boot>) -> Status {
        Status::SUCCESS
    }
}

mod bad_async {
    use super::*;

    #[entry]
    async fn efi_main(_handle: Handle, _st: SystemTable<Boot>) -> Status {
        Status::SUCCESS
    }
}

mod bad_const {
    use super::*;

    #[entry]
    const fn efi_main(_handle: Handle, _st: SystemTable<Boot>) -> Status {
        Status::SUCCESS
    }
}

mod bad_generic {
    use super::*;

    #[entry]
    fn efi_main<T>(_handle: Handle, _st: SystemTable<Boot>) -> Status {
        Status::SUCCESS
    }
}

mod bad_args {
    use super::*;

    #[entry]
    fn efi_main(_handle: Handle, _st: SystemTable<Boot>, _x: usize) -> bool {
        false
    }
}

mod bad_return_type {
    use super::*;

    #[entry]
    fn efi_main(_handle: Handle, _st: SystemTable<Boot>) -> bool {
        false
    }
}
