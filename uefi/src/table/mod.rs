//! Standard UEFI tables.

use core::cell::UnsafeCell;
use core::ptr;

/// Common trait implemented by all standard UEFI tables.
pub trait Table {
    /// A unique number assigned by the UEFI specification
    /// to the standard tables.
    const SIGNATURE: u64;
}

mod header;
pub use self::header::Header;

mod system;
pub use self::system::{Boot, Runtime, SystemTable};

pub mod boot;
pub mod runtime;

pub mod cfg;

pub use uefi_raw::table::Revision;

// TODO: this similar to `SyncUnsafeCell`. Once that is stabilized we
// can use it instead.
struct GlobalSystemTable {
    ptr: UnsafeCell<*mut uefi_raw::table::system::SystemTable>,
}

// Safety: TODO
unsafe impl Sync for GlobalSystemTable {}

static SYSTEM_TABLE: GlobalSystemTable = GlobalSystemTable {
    ptr: UnsafeCell::new(ptr::null_mut()),
};

/// TODO
pub unsafe fn set_system_table(system_table: *mut uefi_raw::table::system::SystemTable) {
    SYSTEM_TABLE.ptr.get().write(system_table);
}

/// TODO
pub fn system_table_boot() -> SystemTable<Boot> {
    unsafe {
        let ptr = SYSTEM_TABLE.ptr.get().read();
        if ptr.is_null() {
            panic!("set_system_table has not been called");
        } else {
            if (*ptr).boot_services.is_null() {
                panic!("boot services are not active");
            } else {
                // OK to unwrap: we've already checked that the pointer is valid.
                SystemTable::<Boot>::from_ptr(ptr.cast()).unwrap()
            }
        }
    }
}

/// TODO
pub(crate) fn boot_services() -> *mut uefi_raw::table::boot::BootServices {
    unsafe {
        let ptr = SYSTEM_TABLE.ptr.get().read();
        if ptr.is_null() {
            panic!("set_system_table has not been called");
        } else {
            let boot_services = (*ptr).boot_services;
            if boot_services.is_null() {
                panic!("boot services are not active");
            } else {
                boot_services
            }
        }
    }
}
