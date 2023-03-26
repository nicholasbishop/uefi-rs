use crate::proto::console::text;
use crate::{Char16, Handle};

use super::boot::BootServices;
use super::runtime::RuntimeServices;
use super::{cfg, Header};

/// The actual UEFI system table
#[repr(C)]
pub struct SystemTable {
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

impl SystemTable {
    pub const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}
