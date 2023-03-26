use core::ffi::c_void;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::slice;

use crate::proto::console::text;
use crate::{CStr16, Char16, Handle, Result};

use super::boot::{BootServices, MemoryDescriptor};
use super::runtime::RuntimeServices;
use super::{cfg, Header, Revision};

/// UEFI System Table interface
///
/// The UEFI System Table is the gateway to all UEFI services which an UEFI
/// application is provided access to on startup. However, not all UEFI services
/// will remain accessible forever.
///
/// Some services, called "boot services", may only be called during a bootstrap
/// stage where the UEFI firmware still has control of the hardware, and will
/// become unavailable once the firmware hands over control of the hardware to
/// an operating system loader. Others, called "runtime services", may still be
/// used after that point, but require a rather specific CPU configuration which
/// an operating system loader is unlikely to preserve.
///
/// We handle this state transition by providing two different views of the UEFI
/// system table, the "Boot" view and the "Runtime" view. An UEFI application
/// is initially provided with access to the "Boot" view, and may transition
/// to the "Runtime" view through the ExitBootServices mechanism that is
/// documented in the UEFI spec. At that point, the boot view of the system
/// table will be destroyed (which conveniently invalidates all references to
/// UEFI boot services in the eye of the Rust borrow checker) and a runtime view
/// will be provided to replace it.
#[derive(Debug)]
#[repr(transparent)]
pub struct SystemTable<View: SystemTableView> {
    table: *const SystemTableImpl,
    _marker: PhantomData<View>,
}

// These parts of the UEFI System Table interface will always be available
impl<View: SystemTableView> SystemTable<View> {
    /// Return the firmware vendor string
    #[must_use]
    pub fn firmware_vendor(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr((*self.table).fw_vendor) }
    }

    /// Return the firmware revision
    #[must_use]
    pub const fn firmware_revision(&self) -> u32 {
        unsafe { (*self.table).fw_revision }
    }

    /// Returns the revision of this table, which is defined to be
    /// the revision of the UEFI specification implemented by the firmware.
    #[must_use]
    pub const fn uefi_revision(&self) -> Revision {
        unsafe { (*self.table).header.revision }
    }

    /// Returns the config table entries, a linear array of structures
    /// pointing to other system-specific tables.
    #[allow(clippy::missing_const_for_fn)] // Required until we bump the MSRV.
    #[must_use]
    pub fn config_table(&self) -> &[cfg::ConfigTableEntry] {
        unsafe { slice::from_raw_parts((*self.table).cfg_table, (*self.table).nr_cfg) }
    }

    /// Creates a new `SystemTable<View>` from a raw address. The address might
    /// come from the Multiboot2 information structure or something similar.
    ///
    /// # Example
    /// ```no_run
    /// use core::ffi::c_void;
    /// use uefi::prelude::{Boot, SystemTable};
    ///
    /// let system_table_addr = 0xdeadbeef as *mut c_void;
    ///
    /// let mut uefi_system_table = unsafe {
    ///     SystemTable::<Boot>::from_ptr(system_table_addr).expect("Pointer must not be null!")
    /// };
    /// ```
    ///
    /// # Safety
    /// This function is unsafe because the caller must be sure that the pointer
    /// is valid. Otherwise, further operations on the object might result in
    /// undefined behaviour, even if the methods aren't marked as unsafe.
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        NonNull::new(ptr.cast()).map(|ptr| Self {
            table: ptr.as_ref(),
            _marker: PhantomData,
        })
    }
}

// These parts of the SystemTable struct are only visible after exit from UEFI
// boot services. They provide unsafe access to the UEFI runtime services, which
// which were already available before but in safe form.
impl SystemTable<Runtime> {
    /// Access runtime services
    ///
    /// # Safety
    ///
    /// This is unsafe because UEFI runtime services require an elaborate
    /// CPU configuration which may not be preserved by OS loaders. See the
    /// "Calling Conventions" chapter of the UEFI specification for details.
    #[must_use]
    pub const unsafe fn runtime_services(&self) -> &RuntimeServices {
        &*(*self.table).runtime
    }

    /// Changes the runtime addressing mode of EFI firmware from physical to virtual.
    /// It is up to the caller to translate the old SystemTable address to a new virtual
    /// address and provide it for this function.
    /// See [`get_current_system_table_addr`]
    ///
    /// # Safety
    ///
    /// Setting new virtual memory map is unsafe and may cause undefined behaviors.
    ///
    /// [`get_current_system_table_addr`]: SystemTable::get_current_system_table_addr
    pub unsafe fn set_virtual_address_map(
        self,
        map: &mut [MemoryDescriptor],
        new_system_table_virtual_addr: u64,
    ) -> Result<Self> {
        // Unsafe Code Guidelines guarantees that there is no padding in an array or a slice
        // between its elements if the element type is `repr(C)`, which is our case.
        //
        // See https://rust-lang.github.io/unsafe-code-guidelines/layout/arrays-and-slices.html
        let map_size = core::mem::size_of_val(map);
        let entry_size = core::mem::size_of::<MemoryDescriptor>();
        let entry_version = crate::table::boot::MEMORY_DESCRIPTOR_VERSION;
        let map_ptr = map.as_mut_ptr();
        ((*(*self.table).runtime).set_virtual_address_map)(
            map_size,
            entry_size,
            entry_version,
            map_ptr,
        )
        .into_with_val(|| {
            let new_table_ref =
                &mut *(new_system_table_virtual_addr as usize as *mut SystemTableImpl);
            Self {
                table: new_table_ref,
                _marker: PhantomData,
            }
        })
    }

    /// Return the address of the SystemTable that resides in a UEFI runtime services
    /// memory region.
    #[must_use]
    pub fn get_current_system_table_addr(&self) -> u64 {
        self.table as u64
    }
}

/// The actual UEFI system table
#[repr(C)]
pub struct SystemTableImpl {
    header: Header,
    /// Null-terminated string representing the firmware's vendor.
    fw_vendor: *const Char16,
    fw_revision: u32,
    stdin_handle: Handle,
    stdin: *mut text::Input,
    stdout_handle: Handle,
    stdout: *mut text::Output,
    stderr_handle: Handle,
    stderr: *mut text::Output,
    /// Runtime services table.
    runtime: *const RuntimeServices,
    /// Boot services table.
    boot: *const BootServices,
    /// Number of entries in the configuration table.
    nr_cfg: usize,
    /// Pointer to beginning of the array.
    cfg_table: *const cfg::ConfigTableEntry,
}

impl<View: SystemTableView> super::Table for SystemTable<View> {
    const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}
