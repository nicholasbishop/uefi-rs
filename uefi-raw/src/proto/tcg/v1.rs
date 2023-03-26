//! [TCG] (Trusted Computing Group) protocol for [TPM] (Trusted Platform
//! Module) 1.1 and 1.2.
//!
//! This protocol is defined in the [TCG EFI Protocol Specification _for
//! TPM Family 1.1 or 1.2_][spec].
//!
//! [spec]: https://trustedcomputinggroup.org/resource/tcg-efi-protocol-specification/
//! [TCG]: https://trustedcomputinggroup.org/
//! [TPM]: https://en.wikipedia.org/wiki/Trusted_Platform_Module

use super::{EventType, HashAlgorithm, PcrIndex};
use crate::data_types::PhysicalAddress;
use crate::proto::unsafe_protocol;
use crate::Status;
use core::fmt::{self, Debug, Formatter};
use core::marker::{PhantomData, PhantomPinned};
use ptr_meta::Pointee;

/// 20-byte SHA-1 digest.
pub type Sha1Digest = [u8; 20];

/// This corresponds to the `AlgorithmId` enum, but in the v1 spec it's `u32`
/// instead of `u16`.
#[allow(non_camel_case_types)]
type TCG_ALGORITHM_ID = u32;

/// Information about the protocol and the TPM device.
///
/// Layout compatible with the C type `TCG_EFI_BOOT_SERVICE_CAPABILITY`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootServiceCapability {
    size: u8,
    structure_version: Version,
    protocol_spec_version: Version,
    hash_algorithm_bitmap: u8,
    tpm_present_flag: u8,
    tpm_deactivated_flag: u8,
}

impl BootServiceCapability {
    /// Version of the `BootServiceCapability` structure.
    #[must_use]
    pub fn structure_version(&self) -> Version {
        self.structure_version
    }

    /// Version of the `Tcg` protocol.
    #[must_use]
    pub fn protocol_spec_version(&self) -> Version {
        self.protocol_spec_version
    }

    /// Supported hash algorithms.
    #[must_use]
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        // Safety: the value should always be 0x1 (indicating SHA-1), but
        // we don't care if it's some unexpected value.
        unsafe { HashAlgorithm::from_bits_unchecked(u32::from(self.hash_algorithm_bitmap)) }
    }

    /// Whether the TPM device is present.
    #[must_use]
    pub fn tpm_present(&self) -> bool {
        self.tpm_present_flag != 0
    }

    /// Whether the TPM device is deactivated.
    #[must_use]
    pub fn tpm_deactivated(&self) -> bool {
        self.tpm_deactivated_flag != 0
    }
}

/// Version information.
///
/// Layout compatible with the C type `TCG_VERSION`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,

    // Leave these two fields undocumented since it's not clear what
    // they are for. The spec doesn't say, and they were removed in the
    // v2 spec.
    #[allow(missing_docs)]
    pub rev_major: u8,
    #[allow(missing_docs)]
    pub rev_minor: u8,
}

/// Entry in the [`EventLog`].
///
/// Layout compatible with the C type `TCG_PCR_EVENT`.
///
/// Naming note: the spec refers to "event data" in two conflicting
/// ways: the `event_data` field and the data hashed in the digest
/// field. These two are independent; although the event data _can_ be
/// what is hashed in the digest field, it doesn't have to be.
#[repr(C, packed)]
#[derive(Pointee)]
pub struct PcrEvent {
    pcr_index: PcrIndex,
    event_type: EventType,
    digest: Sha1Digest,
    event_data_size: u32,
    event_data: [u8],
}

// Manual `Debug` implementation since it can't be derived for a packed DST.
impl Debug for PcrEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcrEvent")
            .field("pcr_index", &{ self.pcr_index })
            .field("event_type", &{ self.event_type })
            .field("digest", &self.digest)
            .field("event_data_size", &{ self.event_data_size })
            .field("event_data", &&self.event_data)
            .finish()
    }
}

/// Opaque type that should be used to represent a pointer to a [`PcrEvent`] in
/// foreign function interfaces. This type produces a thin pointer, unlike
/// [`PcrEvent`].
#[repr(C, packed)]
#[derive(Debug)]
pub struct FfiPcrEvent {
    // This representation is recommended by the nomicon:
    // https://doc.rust-lang.org/stable/nomicon/ffi.html#representing-opaque-structs
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// Protocol for interacting with TPM 1.1 and 1.2 devices.
///
/// The corresponding C type is `EFI_TCG_PROTOCOL`.
#[repr(C)]
#[unsafe_protocol("f541796d-a62e-4954-a775-9584f61b9cdd")]
pub struct Tcg {
    status_check: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        protocol_capability: *mut BootServiceCapability,
        feature_flags: *mut u32,
        event_log_location: *mut PhysicalAddress,
        event_log_last_entry: *mut PhysicalAddress,
    ) -> Status,

    // Note: we do not currently expose this function because the spec
    // for this is not well written. The function allocates memory, but
    // the spec doesn't say how to free it. Most likely
    // `EFI_BOOT_SERVICES.FreePool` would work, but this is not
    // mentioned in the spec so it is unsafe to rely on.
    //
    // Also, this function is not that useful in practice for a couple
    // reasons. First, it takes an algorithm ID, but only SHA-1 is
    // supported with TPM v1. Second, TPMs are not cryptographic
    // accelerators, so it is very likely faster to calculate the hash
    // on the CPU, e.g. with the `sha1` crate.
    hash_all: unsafe extern "efiapi" fn() -> Status,

    log_event: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        // The spec does not guarantee that the `event` will not be mutated
        // through the pointer, but it seems reasonable to assume and makes the
        // public interface clearer, so use a const pointer.
        event: *const FfiPcrEvent,
        event_number: *mut u32,
        flags: u32,
    ) -> Status,

    pass_through_to_tpm: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        tpm_input_parameter_block_size: u32,
        tpm_input_parameter_block: *const u8,
        tpm_output_parameter_block_size: u32,
        tpm_output_parameter_block: *mut u8,
    ) -> Status,

    hash_log_extend_event: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        hash_data: PhysicalAddress,
        hash_data_len: u64,
        algorithm_id: TCG_ALGORITHM_ID,
        event: *mut FfiPcrEvent,
        event_number: *mut u32,
        event_log_last_entry: *mut PhysicalAddress,
    ) -> Status,
}
