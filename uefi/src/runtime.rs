//! UEFI runtime services.
//!
//! These services are available both before and after exiting boot
//! services. Note that various restrictions apply when calling runtime services
//! functions after exiting boot services; see the "Calling Convention" section
//! of the UEFI specification for details.

use crate::data_types::PhysicalAddress;
use crate::table::{self, Revision};
use crate::{CStr16, Error, Result, Status, StatusExt};
use core::ptr::{self, NonNull};

#[cfg(feature = "alloc")]
use {
    crate::mem::make_boxed, crate::Guid, alloc::borrow::ToOwned, alloc::boxed::Box,
    alloc::vec::Vec, core::mem,
};

#[cfg(all(feature = "unstable", feature = "alloc"))]
use alloc::alloc::Global;

pub use crate::table::runtime::{
    CapsuleInfo, Daylight, Time, TimeCapabilities, TimeError, TimeParams, VariableStorageInfo,
};
pub use uefi_raw::capsule::{CapsuleBlockDescriptor, CapsuleFlags, CapsuleHeader};
pub use uefi_raw::table::runtime::{ResetType, VariableAttributes, VariableVendor};

#[cfg(feature = "alloc")]
pub use crate::table::runtime::VariableKey;

fn runtime_services_raw_panicking() -> NonNull<uefi_raw::table::runtime::RuntimeServices> {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    NonNull::new(st.runtime_services).expect("runtime services are not active")
}

/// Query the current time and date information.
pub fn get_time() -> Result<Time> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut time = Time::invalid();
    let time_ptr: *mut Time = &mut time;
    unsafe { (rt.get_time)(time_ptr.cast(), ptr::null_mut()) }.to_result_with_val(|| time)
}

/// Query the current time and date information and the RTC capabilities.
pub fn get_time_and_caps() -> Result<(Time, TimeCapabilities)> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut time = Time::invalid();
    let time_ptr: *mut Time = &mut time;
    let mut caps = TimeCapabilities::default();
    unsafe { (rt.get_time)(time_ptr.cast(), &mut caps) }.to_result_with_val(|| (time, caps))
}

/// Sets the current local time and date information
///
/// During runtime, if a PC-AT CMOS device is present in the platform, the
/// caller must synchronize access to the device before calling `set_time`.
///
/// # Safety
///
/// Undefined behavior could happen if multiple tasks try to
/// use this function at the same time without synchronisation.
pub unsafe fn set_time(time: &Time) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let time: *const Time = time;
    (rt.set_time)(time.cast()).to_result()
}

/// Gets the contents and attributes of a variable. The size of `buf` must be at
/// least as big as the variable's size, although it can be larger.
///
/// On success, returns a tuple containing the variable's value (a slice of
/// `buf`) and the variable's attributes.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: variable was not found.
/// * [`Status::BUFFER_TOO_SMALL`]: `buf` is not large enough. The required size
///   will be returned in the error data.
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be read due to an
///   authentication error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn get_variable<'buf>(
    name: &CStr16,
    vendor: &VariableVendor,
    buf: &'buf mut [u8],
) -> Result<(&'buf mut [u8], VariableAttributes), Option<usize>> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut attributes = VariableAttributes::empty();
    let mut data_size = buf.len();
    let status = unsafe {
        (rt.get_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            &mut attributes,
            &mut data_size,
            buf.as_mut_ptr(),
        )
    };

    match status {
        Status::SUCCESS => Ok((&mut buf[..data_size], attributes)),
        Status::BUFFER_TOO_SMALL => Err(Error::new(status, Some(data_size))),
        _ => Err(Error::new(status, None)),
    }
}

/// Gets the contents and attributes of a variable.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: variable was not found.
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be read due to an
///   authentication error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
#[cfg(feature = "alloc")]
pub fn get_variable_boxed(
    name: &CStr16,
    vendor: &VariableVendor,
) -> Result<(Box<[u8]>, VariableAttributes)> {
    let mut out_attr = VariableAttributes::empty();
    let get_var = |buf| {
        get_variable(name, vendor, buf).map(|(val, attr)| {
            // `make_boxed` expects only a DST value to be returned (`val` in
            // this case), so smuggle the `attr` value out via a separate
            // variable.
            out_attr = attr;
            val
        })
    };
    #[cfg(not(feature = "unstable"))]
    {
        make_boxed(get_var).map(|val| (val, out_attr))
    }
    #[cfg(feature = "unstable")]
    {
        make_boxed(get_var, Global).map(|val| (val, out_attr))
    }
}

/// Get an iterator over all UEFI variables.
#[cfg(feature = "alloc")]
pub fn variable_keys() -> VariableKeys {
    VariableKeys::new()
}

/// Iterator over all UEFI variables.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct VariableKeys {
    name: Vec<u16>,
    vendor: VariableVendor,
    is_done: bool,
}

#[cfg(feature = "alloc")]
impl VariableKeys {
    fn new() -> Self {
        // Create a the name buffer with a reasonable default capacity, and
        // initialize it to an empty null-terminated string.
        let mut name = Vec::with_capacity(32);
        name.push(0);

        Self {
            // Give the name buffer a reasonable default capacity.
            name,
            // The initial vendor GUID is arbitrary.
            vendor: VariableVendor(Guid::default()),
            is_done: false,
        }
    }
}

#[cfg(feature = "alloc")]
impl Iterator for VariableKeys {
    type Item = Result<VariableKey>;

    fn next(&mut self) -> Option<Result<VariableKey>> {
        if self.is_done {
            return None;
        }

        let rt = runtime_services_raw_panicking();
        let rt = unsafe { rt.as_ref() };

        let mut name_size_in_bytes = self.name.len() * mem::size_of::<u16>();

        let mut status = unsafe {
            (rt.get_next_variable_name)(
                &mut name_size_in_bytes,
                self.name.as_mut_ptr(),
                &mut self.vendor.0,
            )
        };

        // If the name buffer passed in was too small, resize it to be big
        // enough and call `get_next_variable_name` again.
        if status == Status::BUFFER_TOO_SMALL {
            self.name
                .resize(name_size_in_bytes / mem::size_of::<u16>(), 0);
            status = unsafe {
                (rt.get_next_variable_name)(
                    &mut name_size_in_bytes,
                    self.name.as_mut_ptr(),
                    &mut self.vendor.0,
                )
            }
        }

        match status {
            Status::SUCCESS => {
                let name = if let Some(nul_pos) = self.name.iter().position(|c| *c == 0) {
                    self.name[..=nul_pos].to_owned()
                } else {
                    self.name.clone()
                };
                Some(Ok(VariableKey {
                    name,
                    vendor: self.vendor,
                }))
            }
            Status::NOT_FOUND => {
                // This status indicates the end of the list. The final variable
                // has already been yielded at this point, so return `None`.
                self.is_done;
                None
            }
            _ => {
                // Return the error and end iteration.
                self.is_done = true;
                Some(Err(Error::from(status)))
            }
        }
    }
}

/// Sets the value of a variable. This can be used to create a new variable,
/// update an existing variable, or (when the size of `data` is zero)
/// delete a variable.
///
/// # Warnings
///
/// The [`Status::WARN_RESET_REQUIRED`] warning will be returned when using
/// this function to transition the Secure Boot mode to setup mode or audit
/// mode if the firmware requires a reboot for that operation.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: invalid attributes, name, or vendor.
/// * [`Status::OUT_OF_RESOURCES`]: not enough storage is available to hold
///   the variable.
/// * [`Status::WRITE_PROTECTED`]: variable is read-only.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be written due to an
///   authentication error.
/// * [`Status::NOT_FOUND`]: attempted to update a non-existent variable.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn set_variable(
    name: &CStr16,
    vendor: &VariableVendor,
    attributes: VariableAttributes,
    data: &[u8],
) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    unsafe {
        (rt.set_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            attributes,
            data.len(),
            data.as_ptr(),
        )
        .to_result()
    }
}

/// Deletes a UEFI variable.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: invalid name or vendor.
/// * [`Status::WRITE_PROTECTED`]: variable is read-only.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be deleted due to an
///   authentication error.
/// * [`Status::NOT_FOUND`]: attempted to delete a non-existent variable.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn delete_variable(name: &CStr16, vendor: &VariableVendor) -> Result {
    set_variable(name, vendor, VariableAttributes::empty(), &[])
}

/// Get information about UEFI variable storage space for the type
/// of variable specified in `attributes`.
///
/// This operation is only supported starting with UEFI 2.0; earlier
/// versions will fail with [`Status::UNSUPPORTED`].
///
/// See [`VariableStorageInfo`] for details of the information returned.
pub fn query_variable_info(attributes: VariableAttributes) -> Result<VariableStorageInfo> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    if rt.header.revision < Revision::EFI_2_00 {
        return Err(Status::UNSUPPORTED.into());
    }

    let mut info = VariableStorageInfo::default();
    unsafe {
        (rt.query_variable_info)(
            attributes,
            &mut info.maximum_variable_storage_size,
            &mut info.remaining_variable_storage_size,
            &mut info.maximum_variable_size,
        )
        .to_result_with_val(|| info)
    }
}

/// Resets the computer.
pub fn reset(reset_type: ResetType, status: Status, data: Option<&[u8]>) -> ! {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let (size, data) = match data {
        // FIXME: The UEFI spec states that the data must start with a NUL-
        //        terminated string, which we should check... but it does not
        //        specify if that string should be Latin-1 or UCS-2!
        //
        //        PlatformSpecific resets should also insert a GUID after the
        //        NUL-terminated string.
        Some(data) => (data.len(), data.as_ptr()),
        None => (0, ptr::null()),
    };

    unsafe { (rt.reset_system)(reset_type, status, size, data) }
}

/// Passes capsules to the firmware. Capsules are most commonly used to update system firmware.
pub fn update_capsule(
    capsule_header_array: &[&CapsuleHeader],
    capsule_block_descriptors: &[CapsuleBlockDescriptor],
) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    unsafe {
        (rt.update_capsule)(
            capsule_header_array.as_ptr().cast(),
            capsule_header_array.len(),
            capsule_block_descriptors.as_ptr() as PhysicalAddress,
        )
        .to_result()
    }
}

/// Tests whether a capsule or capsules can be updated via [`update_capsule`].
///
/// See [`CapsuleInfo`] for details of the information returned.
pub fn query_capsule_capabilities(capsule_header_array: &[&CapsuleHeader]) -> Result<CapsuleInfo> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut info = CapsuleInfo::default();
    unsafe {
        (rt.query_capsule_capabilities)(
            capsule_header_array.as_ptr().cast(),
            capsule_header_array.len(),
            &mut info.maximum_capsule_size,
            &mut info.reset_type,
        )
        .to_result_with_val(|| info)
    }
}
