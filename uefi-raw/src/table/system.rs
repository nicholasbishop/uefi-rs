use crate::protocol::console::text;
use crate::{Char16, Handle};

use super::boot::BootServices;
use super::runtime::RuntimeServices;
use super::{cfg, Header};

/// The actual UEFI system table
#[repr(C)]
pub struct SystemTable {
    pub header: Header,
    /// Null-terminated string representing the firmware's vendor.
    pub fw_vendor: *const Char16,
    pub fw_revision: u32,
    pub stdin_handle: Handle,
    pub stdin: *mut text::Input,
    pub stdout_handle: Handle,
    pub stdout: *mut text::Output,
    pub stderr_handle: Handle,
    pub stderr: *mut text::Output,
    /// Runtime services table.
    pub runtime: *const RuntimeServices,
    /// Boot services table.
    pub boot: *const BootServices,
    /// Number of entries in the configuration table.
    pub nr_cfg: usize,
    /// Pointer to beginning of the array.
    pub cfg_table: *const cfg::ConfigTableEntry,
}

impl SystemTable {
    pub const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}
