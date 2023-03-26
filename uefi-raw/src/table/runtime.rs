//! UEFI services available at runtime, even after the OS boots.

use super::Header;
use crate::table::boot::MemoryDescriptor;
use crate::{guid, Char16, Guid, Status};
use bitflags::bitflags;
use core::fmt;
use core::fmt::{Debug, Formatter};

#[repr(C)]
pub struct RuntimeServices {
    pub header: Header,
    pub get_time:
        unsafe extern "efiapi" fn(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status,
    pub set_time: unsafe extern "efiapi" fn(time: &Time) -> Status,
    pub get_wakeup_time:
        unsafe extern "efiapi" fn(enabled: *mut u8, pending: *mut u8, time: *mut Time) -> Status,
    pub set_wakeup_time: unsafe extern "efiapi" fn(enable: u8, time: *const Time) -> Status,
    pub set_virtual_address_map: unsafe extern "efiapi" fn(
        map_size: usize,
        desc_size: usize,
        desc_version: u32,
        virtual_map: *mut MemoryDescriptor,
    ) -> Status,
    pub pad2: usize,
    pub get_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const Guid,
        attributes: *mut VariableAttributes,
        data_size: *mut usize,
        data: *mut u8,
    ) -> Status,
    pub get_next_variable_name: unsafe extern "efiapi" fn(
        variable_name_size: *mut usize,
        variable_name: *mut u16,
        vendor_guid: *mut Guid,
    ) -> Status,
    pub set_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const Guid,
        attributes: VariableAttributes,
        data_size: usize,
        data: *const u8,
    ) -> Status,
    pub _pad3: usize,
    pub reset: unsafe extern "efiapi" fn(
        rt: ResetType,

        status: Status,
        data_size: usize,
        data: *const u8,
    ) -> !,

    // UEFI 2.0 Capsule Services.
    pub update_capsule: usize,
    pub query_capsule_capabilities: usize,

    // Miscellaneous UEFI 2.0 Service.
    pub query_variable_info: unsafe extern "efiapi" fn(
        attributes: VariableAttributes,
        maximum_variable_storage_size: *mut u64,
        remaining_variable_storage_size: *mut u64,
        maximum_variable_size: *mut u64,
    ) -> Status,
}

impl RuntimeServices {
    pub const SIGNATURE: u64 = 0x5652_4553_544e_5552;
}

impl Debug for RuntimeServices {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeServices")
            .field("header", &self.header)
            .field("get_time", &(self.get_time as *const u64))
            .field("set_time", &(self.set_time as *const u64))
            .field(
                "set_virtual_address_map",
                &(self.set_virtual_address_map as *const u64),
            )
            .field("reset", &(self.reset as *const u64))
            .finish()
    }
}

/// Date and time representation.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Time {
    pub year: u16,  // 1900 - 9999
    pub month: u8,  // 1 - 12
    pub day: u8,    // 1 - 31
    pub hour: u8,   // 0 - 23
    pub minute: u8, // 0 - 59
    pub second: u8, // 0 - 59
    pub _pad1: u8,
    pub nanosecond: u32, // 0 - 999_999_999
    pub time_zone: i16,  // -1440 to 1440, or 2047 if unspecified
    pub daylight: Daylight,
    pub _pad2: u8,
}

bitflags! {
    /// A bitmask containing daylight savings time information.
    pub struct Daylight: u8 {
        /// Time is affected by daylight savings time.
        const ADJUST_DAYLIGHT = 0x01;
        /// Time has been adjusted for daylight savings time.
        const IN_DAYLIGHT = 0x02;
    }
}

/// Error returned by [`Time`] methods if the input is outside the valid range.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct TimeError;

impl Time {
    /// Unspecified Timezone/local time.
    const UNSPECIFIED_TIMEZONE: i16 = 0x07ff;

    /// True if all fields are within valid ranges, false otherwise.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        (1900..=9999).contains(&self.year)
            && (1..=12).contains(&self.month)
            && (1..=31).contains(&self.day)
            && self.hour <= 23
            && self.minute <= 59
            && self.second <= 59
            && self.nanosecond <= 999_999_999
            && ((-1440..=1440).contains(&self.time_zone)
                || self.time_zone == Self::UNSPECIFIED_TIMEZONE)
    }

    /// Query the year.
    #[must_use]
    pub const fn year(&self) -> u16 {
        self.year
    }

    /// Query the month.
    #[must_use]
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// Query the day.
    #[must_use]
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Query the hour.
    #[must_use]
    pub const fn hour(&self) -> u8 {
        self.hour
    }

    /// Query the minute.
    #[must_use]
    pub const fn minute(&self) -> u8 {
        self.minute
    }

    /// Query the second.
    #[must_use]
    pub const fn second(&self) -> u8 {
        self.second
    }

    /// Query the nanosecond.
    #[must_use]
    pub const fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Query the time offset in minutes from UTC, or None if using local time.
    #[must_use]
    pub const fn time_zone(&self) -> Option<i16> {
        if self.time_zone == 2047 {
            None
        } else {
            Some(self.time_zone)
        }
    }

    /// Query the daylight savings time information.
    #[must_use]
    pub const fn daylight(&self) -> Daylight {
        self.daylight
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02} ", self.year, self.month, self.day)?;
        write!(
            f,
            "{:02}:{:02}:{:02}.{:09}",
            self.hour, self.minute, self.second, self.nanosecond
        )?;
        if self.time_zone == Self::UNSPECIFIED_TIMEZONE {
            write!(f, ", Timezone=local")?;
        } else {
            write!(f, ", Timezone={}", self.time_zone)?;
        }
        write!(f, ", Daylight={:?}", self.daylight)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02} ", self.year, self.month, self.day)?;
        write!(
            f,
            "{:02}:{:02}:{:02}.{:09}",
            self.hour, self.minute, self.second, self.nanosecond
        )?;

        if self.time_zone == Self::UNSPECIFIED_TIMEZONE {
            write!(f, " (local)")?;
        } else {
            let offset_in_hours = self.time_zone as f32 / 60.0;
            let integer_part = offset_in_hours as i16;
            // We can't use "offset_in_hours.fract()" because it is part of `std`.
            let fraction_part = offset_in_hours - (integer_part as f32);
            // most time zones
            if fraction_part == 0.0 {
                write!(f, "UTC+{offset_in_hours}")?;
            }
            // time zones with 30min offset (and perhaps other special time zones)
            else {
                write!(f, "UTC+{offset_in_hours:.1}")?;
            }
        }

        Ok(())
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Time) -> bool {
        self.year == other.year
            && self.month == other.month
            && self.day == other.day
            && self.hour == other.hour
            && self.minute == other.minute
            && self.second == other.second
            && self.nanosecond == other.nanosecond
            && self.time_zone == other.time_zone
            && self.daylight == other.daylight
    }
}

impl Eq for Time {}

/// Real time clock capabilities
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct TimeCapabilities {
    /// Reporting resolution of the clock in counts per second. 1 for a normal
    /// PC-AT CMOS RTC device, which reports the time with 1-second resolution.
    pub resolution: u32,

    /// Timekeeping accuracy in units of 1e-6 parts per million.
    pub accuracy: u32,

    /// Whether a time set operation clears the device's time below the
    /// "resolution" reporting level. False for normal PC-AT CMOS RTC devices.
    pub sets_to_zero: bool,
}

bitflags! {
    /// Flags describing the attributes of a variable.
    pub struct VariableAttributes: u32 {
        /// Variable is maintained across a power cycle.
        const NON_VOLATILE = 0x01;

        /// Variable is accessible during the time that boot services are
        /// accessible.
        const BOOTSERVICE_ACCESS = 0x02;

        /// Variable is accessible during the time that runtime services are
        /// accessible.
        const RUNTIME_ACCESS = 0x04;

        /// Variable is stored in the portion of NVR allocated for error
        /// records.
        const HARDWARE_ERROR_RECORD = 0x08;

        /// Deprecated.
        const AUTHENTICATED_WRITE_ACCESS = 0x10;

        /// Variable payload begins with an EFI_VARIABLE_AUTHENTICATION_2
        /// structure.
        const TIME_BASED_AUTHENTICATED_WRITE_ACCESS = 0x20;

        /// This is never set in the attributes returned by
        /// `get_variable`. When passed to `set_variable`, the variable payload
        /// will be appended to the current value of the variable if supported
        /// by the firmware.
        const APPEND_WRITE = 0x40;

        /// Variable payload begins with an EFI_VARIABLE_AUTHENTICATION_3
        /// structure.
        const ENHANCED_AUTHENTICATED_ACCESS = 0x80;
    }
}

newtype_enum! {
    /// Variable vendor GUID. This serves as a namespace for variables to
    /// avoid naming conflicts between vendors. The UEFI specification
    /// defines some special values, and vendors will define their own.
    pub enum VariableVendor: Guid => {
        /// Used to access global variables.
        GLOBAL_VARIABLE = guid!("8be4df61-93ca-11d2-aa0d-00e098032b8c"),

        /// Used to access EFI signature database variables.
        IMAGE_SECURITY_DATABASE = guid!("d719b2cb-3d3a-4596-a3bc-dad00e67656f"),
    }
}

/// Information about UEFI variable storage space returned by
/// [`RuntimeServices::query_variable_info`]. Note that the data here is
/// limited to a specific type of variable (as specified by the
/// `attributes` argument to `query_variable_info`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VariableStorageInfo {
    /// Maximum size in bytes of the storage space available for
    /// variables of the specified type.
    pub maximum_variable_storage_size: u64,

    /// Remaining size in bytes of the storage space available for
    /// variables of the specified type.
    pub remaining_variable_storage_size: u64,

    /// Maximum size of an individual variable of the specified type.
    pub maximum_variable_size: u64,
}

/// The type of system reset.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ResetType {
    /// Resets all the internal circuitry to its initial state.
    ///
    /// This is analogous to power cycling the device.
    Cold = 0,
    /// The processor is reset to its initial state.
    Warm,
    /// The components are powered off.
    Shutdown,
    /// A platform-specific reset type.
    ///
    /// The additional data must be a pointer to
    /// a null-terminated string followed by an UUID.
    PlatformSpecific,
    // SAFETY: This enum is never exposed to the user, but only fed as input to
    //         the firmware. Therefore, unexpected values can never come from
    //         the firmware, and modeling this as a Rust enum seems safe.
}
