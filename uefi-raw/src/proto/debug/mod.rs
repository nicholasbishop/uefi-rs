//! Provides support for the UEFI debugging protocol.
//!
//! This protocol is designed to allow debuggers to query the state of the firmware,
//! as well as set up callbacks for various events.
//!
//! It also defines a Debugport protocol for debugging over serial devices.
//!
//! An example UEFI debugger is Intel's [UDK Debugger Tool][udk].
//!
//! [udk]: https://firmware.intel.com/develop/intel-uefi-tools-and-utilities/intel-uefi-development-kit-debugger-tool

mod context;
mod exception;

use crate::{guid, Guid, Identify, Status};
use core::ffi::c_void;

pub use self::context::SystemContext;
pub use self::exception::ExceptionType;

/// The debugging support protocol allows debuggers to connect to a UEFI machine.
/// It is expected that there will typically be two instances of the EFI Debug Support protocol in the system.
/// One associated with the native processor instruction set (IA-32, x64, ARM, RISC-V, or Itanium processor
/// family), and one for the EFI virtual machine that implements EFI byte code (EBC).
/// While multiple instances of the EFI Debug Support protocol are expected, there must never be more than
/// one for any given instruction set.
///
/// NOTE: OVMF only implements this protocol interface for the virtual EBC processor
#[repr(C)]
pub struct DebugSupport {
    pub isa: ProcessorArch,
    pub get_maximum_processor_index:
        extern "efiapi" fn(this: &mut DebugSupport, max_processor_index: &mut usize) -> Status,
    pub register_periodic_callback: unsafe extern "efiapi" fn(
        this: &mut DebugSupport,
        processor_index: usize,
        periodic_callback: Option<unsafe extern "efiapi" fn(SystemContext)>,
    ) -> Status,
    pub register_exception_callback: unsafe extern "efiapi" fn(
        this: &mut DebugSupport,
        processor_index: usize,
        exception_callback: Option<unsafe extern "efiapi" fn(ExceptionType, SystemContext)>,
        exception_type: ExceptionType,
    ) -> Status,
    pub invalidate_instruction_cache: unsafe extern "efiapi" fn(
        this: &mut DebugSupport,
        processor_index: usize,
        start: *mut c_void,
        length: u64,
    ) -> Status,
}

unsafe impl Identify for DebugSupport {
    const GUID: Guid = guid!("2755590c-6f3c-42fa-9ea4-a3ba543cda25");
}

newtype_enum! {
/// The instruction set architecture of the running processor.
///
/// UEFI can be and has been ported to new CPU architectures in the past,
/// therefore modeling this C enum as a Rust enum (where the compiler must know
/// about every variant in existence) would _not_ be safe.
pub enum ProcessorArch: u32 => {
    /// 32-bit x86 PC
    X86_32      = 0x014C,
    /// 64-bit x86 PC
    X86_64      = 0x8664,
    /// Intel Itanium
    ITANIUM     = 0x200,
    /// UEFI Interpreter bytecode
    EBC         = 0x0EBC,
    /// ARM Thumb / Mixed
    ARM         = 0x01C2,
    /// ARM 64-bit
    AARCH_64    = 0xAA64,
    /// RISC-V 32-bit
    RISCV_32    = 0x5032,
    /// RISC-V 64-bit
    RISCV_64    = 0x5064,
    /// RISC-V 128-bit
    RISCV_128   = 0x5128,
}}
