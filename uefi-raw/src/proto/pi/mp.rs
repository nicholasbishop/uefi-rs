//! Multi-processor management protocols.
//!
//! On any system with more than one logical processor we can categorize them as:
//!
//! * BSP — bootstrap processor, executes modules that are necessary for booting the system
//! * AP — application processor, any processor other than the bootstrap processor
//!
//! This module contains protocols that provide a generalized way of performing the following tasks on these logical processors:
//!
//! * retrieving information of multi-processor environment and MP-related status of specific processors
//! * dispatching user-provided function to APs
//! * maintaining MP-related processor status

use crate::{guid, Guid, Identify, Status};
use bitflags::bitflags;
use core::ffi::c_void;

/// Callback to be called on the AP.
pub type Procedure = extern "efiapi" fn(*mut c_void);

bitflags! {
    /// Flags indicating if the processor is BSP or AP,
    /// if the processor is enabled or disabled, and if
    /// the processor is healthy.
    #[derive(Default)]
    #[repr(transparent)]
    pub struct StatusFlag: u32 {
        /// Processor is playing the role of BSP.
        const PROCESSOR_AS_BSP_BIT = 1;
        /// Processor is enabled.
        const PROCESSOR_ENABLED_BIT = 1 << 1;
        /// Processor is healthy.
        const PROCESSOR_HEALTH_STATUS_BIT = 1 << 2;
    }
}

/// Information about processor on the platform.
#[repr(C)]
#[derive(Default, Debug)]
pub struct ProcessorInformation {
    /// Unique processor ID determined by system hardware.
    pub processor_id: u64,
    /// Flags indicating BSP, enabled and healthy status.
    pub status_flag: StatusFlag,
    /// Physical location of the processor.
    pub location: CpuPhysicalLocation,
}

/// Information about physical location of the processor.
#[repr(C)]
#[derive(Default, Debug)]
pub struct CpuPhysicalLocation {
    /// Zero-based physical package number that identifies
    /// the cartridge of the processor.
    pub package: u32,
    /// Zero-based physical core number within package of the processor.
    pub core: u32,
    /// Zero-based logical thread number within core of the processor.
    pub thread: u32,
}

/// Protocol that provides services needed for multi-processor management.
#[repr(C)]
pub struct MpServices {
    pub get_number_of_processors: extern "efiapi" fn(
        this: *const MpServices,
        number_of_processors: *mut usize,
        number_of_enabled_processors: *mut usize,
    ) -> Status,
    pub get_processor_info: extern "efiapi" fn(
        this: *const MpServices,
        processor_number: usize,
        processor_info_buffer: *mut ProcessorInformation,
    ) -> Status,
    pub startup_all_aps: extern "efiapi" fn(
        this: *const MpServices,
        procedure: Procedure,
        single_thread: bool,
        wait_event: *mut c_void,
        timeout_in_micro_seconds: usize,
        procedure_argument: *mut c_void,
        failed_cpu_list: *mut *mut usize,
    ) -> Status,
    pub startup_this_ap: extern "efiapi" fn(
        this: *const MpServices,
        procedure: Procedure,
        processor_number: usize,
        wait_event: *mut c_void,
        timeout_in_micro_seconds: usize,
        procedure_argument: *mut c_void,
        finished: *mut bool,
    ) -> Status,
    pub switch_bsp: extern "efiapi" fn(
        this: *const MpServices,
        processor_number: usize,
        enable_old_bsp: bool,
    ) -> Status,
    pub enable_disable_ap: extern "efiapi" fn(
        this: *const MpServices,
        processor_number: usize,
        enable_ap: bool,
        health_flag: *const u32,
    ) -> Status,
    pub who_am_i:
        extern "efiapi" fn(this: *const MpServices, processor_number: *mut usize) -> Status,
}

unsafe impl Identify for MpServices {
    const GUID: Guid = guid!("3fdda605-a76e-4f46-ad29-12f4531b3d08");
}
