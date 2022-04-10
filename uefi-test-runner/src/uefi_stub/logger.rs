// TODO, copying from uefi-services for now

use core::ffi::c_void;
use core::ptr::NonNull;
use uefi::table::boot::{EventType, Tpl};
use uefi::table::{Boot, SystemTable};
use uefi::Event;

/// Global logger object
static mut LOGGER: Option<uefi::logger::Logger> = None;

/// Set up logging
///
/// This is unsafe because you must arrange for the logger to be reset with
/// disable() on exit from UEFI boot services.
pub unsafe fn init(st: &mut SystemTable<Boot>) {
    // Construct the logger.
    let logger = {
        LOGGER = Some(uefi::logger::Logger::new(st.stdout_ptr()));
        LOGGER.as_ref().unwrap()
    };

    // Set the logger.
    log::set_logger(logger).unwrap(); // Can only fail if already initialized.

    // Set logger max level to level specified by log features
    log::set_max_level(log::STATIC_MAX_LEVEL);

    // Schedule logger to be disabled on exit from UEFI boot services
    st.boot_services()
        .create_event(
            EventType::SIGNAL_EXIT_BOOT_SERVICES,
            Tpl::NOTIFY,
            Some(exit_boot_services),
            None,
        )
        .unwrap();
}

/// Notify the utility library that boot services are not safe to call anymore
/// As this is a callback, it must be `extern "efiapi"`.
unsafe extern "efiapi" fn exit_boot_services(_e: Event, _ctx: Option<NonNull<c_void>>) {
    if let Some(ref mut logger) = LOGGER {
        logger.disable();
    }
}
