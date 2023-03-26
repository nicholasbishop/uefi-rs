//! [TCG] (Trusted Computing Group) protocol for [TPM] (Trusted Platform
//! Module) 2.0.
//!
//! This protocol is defined in the [TCG EFI Protocol Specification _TPM
//! Family 2.0_][spec]. It is generally implemented only for TPM 2.0
//! devices, but the spec indicates it can also be used for older TPM
//! devices.
//!
//! [spec]: https://trustedcomputinggroup.org/resource/tcg-efi-protocol-specification/
//! [TCG]: https://trustedcomputinggroup.org/
//! [TPM]: https://en.wikipedia.org/wiki/Trusted_Platform_Module

use super::{AlgorithmId, EventType, HashAlgorithm, PcrIndex};
use crate::proto::unsafe_protocol;
use crate::PhysicalAddress;
use crate::Status;
use bitflags::bitflags;
use core::fmt::{self, Debug, Formatter};
use core::mem;
use ptr_meta::Pointee;

/// Version information.
///
/// Layout compatible with the C type `EFI_TG2_VERSION`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
}

bitflags! {
    /// Event log formats supported by the firmware.
    ///
    /// Corresponds to the C typedef `EFI_TCG2_EVENT_ALGORITHM_BITMAP`.
    #[derive(Default)]
    #[repr(transparent)]
    pub struct EventLogFormat: u32 {
        /// Firmware supports the SHA-1 log format.
        const TCG_1_2 = 0x0000_0001;

        /// Firmware supports the crypto-agile log format.
        const TCG_2 = 0x0000_0002;
    }
}

/// Information about the protocol and the TPM device.
///
/// Layout compatible with the C type `EFI_TCG2_BOOT_SERVICE_CAPABILITY`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootServiceCapability {
    pub size: u8,

    /// Version of the EFI TCG2 protocol.
    pub structure_version: Version,

    /// Version of the EFI TCG2 protocol.
    pub protocol_version: Version,

    /// Bitmap of supported hash algorithms.
    pub hash_algorithm_bitmap: HashAlgorithm,

    /// Event log formats supported by the firmware.
    pub supported_event_logs: EventLogFormat,

    pub present_flag: u8,

    /// Maximum size (in bytes) of a command that can be sent to the TPM.
    pub max_command_size: u16,

    /// Maximum size (in bytes) of a response that can be provided by the TPM.
    pub max_response_size: u16,

    /// Manufacturer ID.
    ///
    /// See the [TCG Vendor ID registry].
    ///
    /// [TCG Vendor ID registry]: https://trustedcomputinggroup.org/resource/vendor-id-registry/
    pub manufacturer_id: u32,

    /// Maximum number of supported PCR banks (hashing algorithms).
    pub number_of_pcr_banks: u32,

    /// Bitmap of currently-active PCR banks (hashing algorithms). This
    /// is a subset of the supported algorithms in [`hash_algorithm_bitmap`].
    ///
    /// [`hash_algorithm_bitmap`]: Self::hash_algorithm_bitmap
    pub active_pcr_banks: HashAlgorithm,
}

impl Default for BootServiceCapability {
    fn default() -> Self {
        // OK to unwrap, the size is less than u8.
        let struct_size = u8::try_from(mem::size_of::<BootServiceCapability>()).unwrap();

        Self {
            size: struct_size,
            structure_version: Version::default(),
            protocol_version: Version::default(),
            hash_algorithm_bitmap: HashAlgorithm::default(),
            supported_event_logs: EventLogFormat::default(),
            present_flag: 0,
            max_command_size: 0,
            max_response_size: 0,
            manufacturer_id: 0,
            number_of_pcr_banks: 0,
            active_pcr_banks: HashAlgorithm::default(),
        }
    }
}

impl BootServiceCapability {
    /// Whether the TPM device is present.
    #[must_use]
    pub fn tpm_present(&self) -> bool {
        self.present_flag != 0
    }
}

bitflags! {
    /// Flags for the [`Tcg::hash_log_extend_event`] function.
    #[derive(Default)]
    #[repr(transparent)]
    pub struct HashLogExtendEventFlags: u64 {
        /// Extend an event but don't log it.
        const EFI_TCG2_EXTEND_ONLY = 0x0000_0000_0000_0001;

        /// Use when measuring a PE/COFF image.
        const PE_COFF_IMAGE = 0x0000_0000_0000_0010;
    }
}

/// Header used in [`PcrEventInputs`].
///
/// Layout compatible with the C type `EFI_TCG2_EVENT_HEADER`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct EventHeader {
    pub header_size: u32,
    pub header_version: u16,
    pub pcr_index: PcrIndex,
    pub event_type: EventType,
}

/// Event type passed to [`Tcg::hash_log_extend_event`].
///
/// Layout compatible with the C type `EFI_TCG2_EVENT`.
///
/// The TPM v1 spec uses a single generic event type for both creating a
/// new event and reading an event from the log. The v2 spec splits this
/// into two structs: `EFI_TCG2_EVENT` for creating events, and
/// `TCG_PCR_EVENT2` for reading events. To help clarify the usage, our
/// API renames these types to `PcrEventInputs` and `PcrEvent`,
/// respectively.
#[derive(Pointee)]
#[repr(C, packed)]
pub struct PcrEventInputs {
    pub size: u32,
    pub event_header: EventHeader,
    pub event: [u8],
}

impl Debug for PcrEventInputs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcrEventInputs")
            .field("size", &{ self.size })
            .field("event_header", &self.event_header)
            .field("event", &"<binary data>")
            .finish()
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlgorithmDigestSize {
    pub algorithm_id: AlgorithmId,
    pub digest_size: u16,
}

/// Header stored at the beginning of the event log.
///
/// Layout compatible with the C type `TCG_EfiSpecIDEventStruct`.
pub struct EventLogHeader {
    pub platform_class: u32,
    // major, minor, errata
    pub spec_version: (u8, u8, u8),
    pub uintn_size: u8,

    // TODO
    pub data: [u8],
}

/// This type of event log can contain multiple hash types (e.g. SHA-1, SHA-256,
/// SHA-512, etc).
#[derive(Debug)]
pub struct EventLog {
    pub location: *const u8,
    pub last_entry: *const u8,

    pub is_truncated: bool,
}

/// Protocol for interacting with TPM devices.
///
/// This protocol can be used for interacting with older TPM 1.1/1.2
/// devices, but most firmware only uses it for TPM 2.0.
///
/// The corresponding C type is `EFI_TCG2_PROTOCOL`.
#[repr(C)]
#[unsafe_protocol("607f766c-7455-42be-930b-e4d76db2720f")]
pub struct Tcg {
    pub get_capability: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        protocol_capability: *mut BootServiceCapability,
    ) -> Status,

    pub get_event_log: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        event_log_format: EventLogFormat,
        event_log_location: *mut PhysicalAddress,
        event_log_last_entry: *mut PhysicalAddress,
        event_log_truncated: *mut u8,
    ) -> Status,

    pub hash_log_extend_event: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        flags: HashLogExtendEventFlags,
        data_to_hash: PhysicalAddress,
        data_to_hash_len: u64,
        // Use `()` here rather than `PcrEventInputs` so that it's a
        // thin pointer.
        event: *const (),
    ) -> Status,

    pub submit_command: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        input_parameter_block_size: u32,
        input_parameter_block: *const u8,
        output_parameter_block_size: u32,
        output_parameter_block: *mut u8,
    ) -> Status,

    pub get_active_pcr_banks:
        unsafe extern "efiapi" fn(this: *mut Tcg, active_pcr_banks: *mut HashAlgorithm) -> Status,

    pub set_active_pcr_banks:
        unsafe extern "efiapi" fn(this: *mut Tcg, active_pcr_banks: HashAlgorithm) -> Status,

    pub get_result_of_set_active_pcr_banks: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        operation_present: *mut u32,
        response: *mut u32,
    ) -> Status,
}
