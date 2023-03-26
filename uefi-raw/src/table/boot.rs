//! UEFI services available during boot.

use super::Header;
use crate::data_types::{Align, PhysicalAddress, VirtualAddress};
use crate::proto::device_path::FfiDevicePath;
use crate::proto::ProtocolPointer;
#[cfg(feature = "alloc")]
use crate::proto::{loaded_image::LoadedImage, media::fs::SimpleFileSystem};
use crate::{Char16, Event, Guid, Handle, Status};
#[cfg(feature = "alloc")]
use ::alloc::vec::Vec;
use bitflags::bitflags;
use core::ffi::c_void;
use core::fmt::Debug;
use core::mem::{self, MaybeUninit};
use core::ptr;
use core::ptr::NonNull;

/// Size in bytes of a UEFI page.
///
/// Note that this is not necessarily the processor's page size. The UEFI page
/// size is always 4 KiB.
pub const PAGE_SIZE: usize = 4096;

#[repr(C)]
pub struct BootServices {
    header: Header,

    // Task Priority services
    raise_tpl: unsafe extern "efiapi" fn(new_tpl: Tpl) -> Tpl,
    restore_tpl: unsafe extern "efiapi" fn(old_tpl: Tpl),

    // Memory allocation functions
    allocate_pages: extern "efiapi" fn(
        alloc_ty: u32,
        mem_ty: MemoryType,
        count: usize,
        addr: &mut PhysicalAddress,
    ) -> Status,
    free_pages: extern "efiapi" fn(addr: PhysicalAddress, pages: usize) -> Status,
    get_memory_map: unsafe extern "efiapi" fn(
        size: &mut usize,
        map: *mut MemoryDescriptor,
        key: &mut MemoryMapKey,
        desc_size: &mut usize,
        desc_version: &mut u32,
    ) -> Status,
    allocate_pool:
        extern "efiapi" fn(pool_type: MemoryType, size: usize, buffer: &mut *mut u8) -> Status,
    free_pool: extern "efiapi" fn(buffer: *mut u8) -> Status,

    // Event & timer functions
    create_event: unsafe extern "efiapi" fn(
        ty: EventType,
        notify_tpl: Tpl,
        notify_func: Option<EventNotifyFn>,
        notify_ctx: Option<NonNull<c_void>>,
        out_event: *mut Event,
    ) -> Status,
    set_timer: unsafe extern "efiapi" fn(event: Event, ty: u32, trigger_time: u64) -> Status,
    wait_for_event: unsafe extern "efiapi" fn(
        number_of_events: usize,
        events: *mut Event,
        out_index: *mut usize,
    ) -> Status,
    signal_event: extern "efiapi" fn(event: Event) -> Status,
    close_event: unsafe extern "efiapi" fn(event: Event) -> Status,
    check_event: unsafe extern "efiapi" fn(event: Event) -> Status,

    // Protocol handlers
    install_protocol_interface: unsafe extern "efiapi" fn(
        handle: &mut Option<Handle>,
        guid: &Guid,
        interface_type: InterfaceType,
        interface: *mut c_void,
    ) -> Status,
    reinstall_protocol_interface: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: &Guid,
        old_interface: *mut c_void,
        new_interface: *mut c_void,
    ) -> Status,
    uninstall_protocol_interface: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: &Guid,
        interface: *mut c_void,
    ) -> Status,
    #[deprecated = "open_protocol and open_protocol_exclusive are better alternatives and available since EFI 1.10 (2002)"]
    handle_protocol:
        extern "efiapi" fn(handle: Handle, proto: &Guid, out_proto: &mut *mut c_void) -> Status,
    _reserved: usize,
    register_protocol_notify: extern "efiapi" fn(
        protocol: &Guid,
        event: Event,
        registration: *mut ProtocolSearchKey,
    ) -> Status,
    locate_handle: unsafe extern "efiapi" fn(
        search_ty: i32,
        proto: Option<&Guid>,
        key: Option<ProtocolSearchKey>,
        buf_sz: &mut usize,
        buf: *mut MaybeUninit<Handle>,
    ) -> Status,
    locate_device_path: unsafe extern "efiapi" fn(
        proto: &Guid,
        device_path: &mut *const FfiDevicePath,
        out_handle: &mut MaybeUninit<Handle>,
    ) -> Status,
    install_configuration_table: usize,

    // Image services
    load_image: unsafe extern "efiapi" fn(
        boot_policy: u8,
        parent_image_handle: Handle,
        device_path: *const FfiDevicePath,
        source_buffer: *const u8,
        source_size: usize,
        image_handle: &mut MaybeUninit<Handle>,
    ) -> Status,
    start_image: unsafe extern "efiapi" fn(
        image_handle: Handle,
        exit_data_size: *mut usize,
        exit_data: &mut *mut Char16,
    ) -> Status,
    exit: extern "efiapi" fn(
        image_handle: Handle,
        exit_status: Status,
        exit_data_size: usize,
        exit_data: *mut Char16,
    ) -> !,
    unload_image: extern "efiapi" fn(image_handle: Handle) -> Status,
    exit_boot_services:
        unsafe extern "efiapi" fn(image_handle: Handle, map_key: MemoryMapKey) -> Status,

    // Misc services
    get_next_monotonic_count: usize,
    stall: extern "efiapi" fn(microseconds: usize) -> Status,
    set_watchdog_timer: unsafe extern "efiapi" fn(
        timeout: usize,
        watchdog_code: u64,
        data_size: usize,
        watchdog_data: *const u16,
    ) -> Status,

    // Driver support services
    connect_controller: unsafe extern "efiapi" fn(
        controller: Handle,
        driver_image: Option<Handle>,
        remaining_device_path: *const FfiDevicePath,
        recursive: bool,
    ) -> Status,
    disconnect_controller: unsafe extern "efiapi" fn(
        controller: Handle,
        driver_image: Option<Handle>,
        child: Option<Handle>,
    ) -> Status,

    // Protocol open / close services
    open_protocol: extern "efiapi" fn(
        handle: Handle,
        protocol: &Guid,
        interface: &mut *mut c_void,
        agent_handle: Handle,
        controller_handle: Option<Handle>,
        attributes: u32,
    ) -> Status,
    close_protocol: extern "efiapi" fn(
        handle: Handle,
        protocol: &Guid,
        agent_handle: Handle,
        controller_handle: Option<Handle>,
    ) -> Status,
    open_protocol_information: usize,

    // Library services
    protocols_per_handle: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol_buffer: *mut *mut *const Guid,
        protocol_buffer_count: *mut usize,
    ) -> Status,
    locate_handle_buffer: unsafe extern "efiapi" fn(
        search_ty: i32,
        proto: Option<&Guid>,
        key: Option<ProtocolSearchKey>,
        no_handles: &mut usize,
        buf: &mut *mut Handle,
    ) -> Status,
    #[deprecated = "open_protocol and open_protocol_exclusive are better alternatives and available since EFI 1.10 (2002)"]
    locate_protocol: extern "efiapi" fn(
        proto: &Guid,
        registration: *mut c_void,
        out_proto: &mut *mut c_void,
    ) -> Status,
    install_multiple_protocol_interfaces: usize,
    uninstall_multiple_protocol_interfaces: usize,

    // CRC services
    calculate_crc32: usize,

    // Misc services
    copy_mem: unsafe extern "efiapi" fn(dest: *mut u8, src: *const u8, len: usize),
    set_mem: unsafe extern "efiapi" fn(buffer: *mut u8, len: usize, value: u8),

    // New event functions (UEFI 2.0 or newer)
    create_event_ex: unsafe extern "efiapi" fn(
        ty: EventType,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotifyFn>,
        notify_ctx: Option<NonNull<c_void>>,
        event_group: Option<NonNull<Guid>>,
        out_event: *mut Event,
    ) -> Status,
}

impl super::Table for BootServices {
    const SIGNATURE: u64 = 0x5652_4553_544f_4f42;
}

newtype_enum! {
/// Task priority level.
///
/// Although the UEFI specification repeatedly states that only the variants
/// specified below should be used in application-provided input, as the other
/// are reserved for internal firmware use, it might still happen that the
/// firmware accidentally discloses one of these internal TPLs to us.
///
/// Since feeding an unexpected variant to a Rust enum is UB, this means that
/// this C enum must be interfaced via the newtype pattern.
pub enum Tpl: usize => {
    /// Normal task execution level.
    APPLICATION = 4,
    /// Async interrupt-style callbacks run at this TPL.
    CALLBACK    = 8,
    /// Notifications are masked at this level.
    ///
    /// This is used in critical sections of code.
    NOTIFY      = 16,
    /// Highest priority level.
    ///
    /// Even processor interrupts are disable at this level.
    HIGH_LEVEL  = 31,
}}

// OpenProtocolAttributes is safe to model as a regular enum because it
// is only used as an input. The attributes are bitflags, but all valid
// combinations are listed in the spec and only ByDriver and Exclusive
// can actually be combined.
//
// Some values intentionally excluded:
//
// ByHandleProtocol (0x01) excluded because it is only intended to be
// used in an implementation of `HandleProtocol`.
//
// TestProtocol (0x04) excluded because it doesn't actually open the
// protocol, just tests if it's present on the handle. Since that
// changes the interface significantly, that's exposed as a separate
// method: `BootServices::test_protocol`.

/// Type of allocation to perform.
#[derive(Debug, Copy, Clone)]
pub enum AllocateType {
    /// Allocate any possible pages.
    AnyPages,
    /// Allocate pages at any address below the given address.
    MaxAddress(PhysicalAddress),
    /// Allocate pages at the specified address.
    Address(PhysicalAddress),
}

newtype_enum! {
/// The type of a memory range.
///
/// UEFI allows firmwares and operating systems to introduce new memory types
/// in the 0x70000000..0xFFFFFFFF range. Therefore, we don't know the full set
/// of memory types at compile time, and it is _not_ safe to model this C enum
/// as a Rust enum.
pub enum MemoryType: u32 => {
    /// This enum variant is not used.
    RESERVED                =  0,
    /// The code portions of a loaded UEFI application.
    LOADER_CODE             =  1,
    /// The data portions of a loaded UEFI applications,
    /// as well as any memory allocated by it.
    LOADER_DATA             =  2,
    /// Code of the boot drivers.
    ///
    /// Can be reused after OS is loaded.
    BOOT_SERVICES_CODE      =  3,
    /// Memory used to store boot drivers' data.
    ///
    /// Can be reused after OS is loaded.
    BOOT_SERVICES_DATA      =  4,
    /// Runtime drivers' code.
    RUNTIME_SERVICES_CODE   =  5,
    /// Runtime services' code.
    RUNTIME_SERVICES_DATA   =  6,
    /// Free usable memory.
    CONVENTIONAL            =  7,
    /// Memory in which errors have been detected.
    UNUSABLE                =  8,
    /// Memory that holds ACPI tables.
    /// Can be reclaimed after they are parsed.
    ACPI_RECLAIM            =  9,
    /// Firmware-reserved addresses.
    ACPI_NON_VOLATILE       = 10,
    /// A region used for memory-mapped I/O.
    MMIO                    = 11,
    /// Address space used for memory-mapped port I/O.
    MMIO_PORT_SPACE         = 12,
    /// Address space which is part of the processor.
    PAL_CODE                = 13,
    /// Memory region which is usable and is also non-volatile.
    PERSISTENT_MEMORY       = 14,
}}

impl MemoryType {
    /// Construct a custom `MemoryType`. Values in the range `0x80000000..=0xffffffff` are free for use if you are
    /// an OS loader.
    #[must_use]
    pub const fn custom(value: u32) -> MemoryType {
        assert!(value >= 0x80000000);
        MemoryType(value)
    }
}

/// Memory descriptor version number
pub const MEMORY_DESCRIPTOR_VERSION: u32 = 1;

/// A structure describing a region of memory.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of memory occupying this range.
    pub ty: MemoryType,
    /// Starting physical address.
    pub phys_start: PhysicalAddress,
    /// Starting virtual address.
    pub virt_start: VirtualAddress,
    /// Number of 4 KiB pages contained in this range.
    pub page_count: u64,
    /// The capability attributes of this memory range.
    pub att: MemoryAttribute,
}

impl Default for MemoryDescriptor {
    fn default() -> MemoryDescriptor {
        MemoryDescriptor {
            ty: MemoryType::RESERVED,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            att: MemoryAttribute::empty(),
        }
    }
}

impl Align for MemoryDescriptor {
    fn alignment() -> usize {
        mem::align_of::<Self>()
    }
}

bitflags! {
    /// Flags describing the capabilities of a memory range.
    pub struct MemoryAttribute: u64 {
        /// Supports marking as uncacheable.
        const UNCACHEABLE = 0x1;
        /// Supports write-combining.
        const WRITE_COMBINE = 0x2;
        /// Supports write-through.
        const WRITE_THROUGH = 0x4;
        /// Support write-back.
        const WRITE_BACK = 0x8;
        /// Supports marking as uncacheable, exported and
        /// supports the "fetch and add" semaphore mechanism.
        const UNCACHABLE_EXPORTED = 0x10;
        /// Supports write-protection.
        const WRITE_PROTECT = 0x1000;
        /// Supports read-protection.
        const READ_PROTECT = 0x2000;
        /// Supports disabling code execution.
        const EXECUTE_PROTECT = 0x4000;
        /// Persistent memory.
        const NON_VOLATILE = 0x8000;
        /// This memory region is more reliable than other memory.
        const MORE_RELIABLE = 0x10000;
        /// This memory range can be set as read-only.
        const READ_ONLY = 0x20000;
        /// This memory is earmarked for specific purposes such as for specific
        /// device drivers or applications. This serves as a hint to the OS to
        /// avoid this memory for core OS data or code that cannot be relocated.
        const SPECIAL_PURPOSE = 0x4_0000;
        /// This memory region is capable of being protected with the CPU's memory
        /// cryptography capabilities.
        const CPU_CRYPTO = 0x8_0000;
        /// This memory must be mapped by the OS when a runtime service is called.
        const RUNTIME = 0x8000_0000_0000_0000;
        /// This memory region is described with additional ISA-specific memory
        /// attributes as specified in `MemoryAttribute::ISA_MASK`.
        const ISA_VALID = 0x4000_0000_0000_0000;
        /// These bits are reserved for describing optional ISA-specific cache-
        /// ability attributes that are not covered by the standard UEFI Memory
        /// Attribute cacheability bits such as `UNCACHEABLE`, `WRITE_COMBINE`,
        /// `WRITE_THROUGH`, `WRITE_BACK`, and `UNCACHEABLE_EXPORTED`.
        ///
        /// See Section 2.3 "Calling Conventions" in the UEFI Specification
        /// for further information on each ISA that takes advantage of this.
        const ISA_MASK = 0x0FFF_F000_0000_0000;
    }
}

/// A unique identifier of a memory map.
///
/// If the memory map changes, this value is no longer valid.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryMapKey(usize);

/// A structure containing the size of a memory descriptor and the size of the
/// memory map.
#[derive(Debug)]
pub struct MemoryMapSize {
    /// Size of a single memory descriptor in bytes
    pub entry_size: usize,
    /// Size of the entire memory map in bytes
    pub map_size: usize,
}

/// An iterator of [`MemoryDescriptor`] that is always associated with the
/// unique [`MemoryMapKey`] contained in the struct.
///
/// To iterate over the entries, call [`MemoryMap::entries`]. To get a sorted
/// map, you manually have to call [`MemoryMap::sort`] first.
#[derive(Debug)]
pub struct MemoryMap<'buf> {
    key: MemoryMapKey,
    buf: &'buf mut [u8],
    entry_size: usize,
    len: usize,
}

impl<'buf> MemoryMap<'buf> {
    #[must_use]
    /// Returns the unique [`MemoryMapKey`] associated with the memory map.
    pub fn key(&self) -> MemoryMapKey {
        self.key
    }

    /// Sorts the memory map by physical address in place.
    /// This operation is optional and should be invoked only once.
    pub fn sort(&mut self) {
        unsafe {
            self.qsort(0, self.len - 1);
        }
    }

    /// Hoare partition scheme for quicksort.
    /// Must be called with `low` and `high` being indices within bounds.
    unsafe fn qsort(&mut self, low: usize, high: usize) {
        if low >= high {
            return;
        }

        let p = self.partition(low, high);
        self.qsort(low, p);
        self.qsort(p + 1, high);
    }

    unsafe fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.get_element_phys_addr(low + (high - low) / 2);

        let mut left_index = low.wrapping_sub(1);
        let mut right_index = high.wrapping_add(1);

        loop {
            while {
                left_index = left_index.wrapping_add(1);

                self.get_element_phys_addr(left_index) < pivot
            } {}

            while {
                right_index = right_index.wrapping_sub(1);

                self.get_element_phys_addr(right_index) > pivot
            } {}

            if left_index >= right_index {
                return right_index;
            }

            self.swap(left_index, right_index);
        }
    }

    /// Indices must be smaller than len.
    unsafe fn swap(&mut self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }

        let base = self.buf.as_mut_ptr();

        unsafe {
            ptr::swap_nonoverlapping(
                base.add(index1 * self.entry_size),
                base.add(index2 * self.entry_size),
                self.entry_size,
            );
        }
    }

    fn get_element_phys_addr(&self, index: usize) -> PhysicalAddress {
        let offset = index.checked_mul(self.entry_size).unwrap();
        let elem = unsafe { &*self.buf.as_ptr().add(offset).cast::<MemoryDescriptor>() };
        elem.phys_start
    }

    /// Returns an iterator over the contained memory map. To get a sorted map,
    /// call [`MemoryMap::sort`] first.
    #[must_use]
    pub fn entries(&self) -> MemoryMapIter {
        MemoryMapIter {
            buffer: self.buf,
            entry_size: self.entry_size,
            index: 0,
            len: self.len,
        }
    }
}

/// An iterator of [`MemoryDescriptor`]. The underlying memory map is always
/// associated with a unique [`MemoryMapKey`].
#[derive(Debug, Clone)]
pub struct MemoryMapIter<'buf> {
    buffer: &'buf [u8],
    entry_size: usize,
    index: usize,
    len: usize,
}

impl<'buf> Iterator for MemoryMapIter<'buf> {
    type Item = &'buf MemoryDescriptor;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.len - self.index;

        (sz, Some(sz))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let descriptor = unsafe {
                &*self
                    .buffer
                    .as_ptr()
                    .add(self.entry_size * self.index)
                    .cast::<MemoryDescriptor>()
            };

            self.index += 1;

            Some(descriptor)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for MemoryMapIter<'_> {
    fn len(&self) -> usize {
        self.len
    }
}

/// The type of handle search to perform.
#[derive(Debug, Copy, Clone)]
pub enum SearchType<'guid> {
    /// Return all handles present on the system.
    AllHandles,
    /// Returns all handles supporting a certain protocol, specified by its GUID.
    ///
    /// If the protocol implements the `Protocol` interface,
    /// you can use the `from_proto` function to construct a new `SearchType`.
    ByProtocol(&'guid Guid),
    /// Return all handles that implement a protocol when an interface for that protocol
    /// is (re)installed.
    ByRegisterNotify(ProtocolSearchKey),
}

impl<'guid> SearchType<'guid> {
    /// Constructs a new search type for a specified protocol.
    #[must_use]
    pub const fn from_proto<P: ProtocolPointer + ?Sized>() -> Self {
        SearchType::ByProtocol(&P::GUID)
    }
}

bitflags! {
    /// Flags describing the type of an UEFI event and its attributes.
    pub struct EventType: u32 {
        /// The event is a timer event and may be passed to `BootServices::set_timer()`
        /// Note that timers only function during boot services time.
        const TIMER = 0x8000_0000;

        /// The event is allocated from runtime memory.
        /// This must be done if the event is to be signaled after ExitBootServices.
        const RUNTIME = 0x4000_0000;

        /// Calling wait_for_event or check_event will enqueue the notification
        /// function if the event is not already in the signaled state.
        /// Mutually exclusive with `NOTIFY_SIGNAL`.
        const NOTIFY_WAIT = 0x0000_0100;

        /// The notification function will be enqueued when the event is signaled
        /// Mutually exclusive with `NOTIFY_WAIT`.
        const NOTIFY_SIGNAL = 0x0000_0200;

        /// The event will be signaled at ExitBootServices time.
        /// This event type should not be combined with any other.
        /// Its notification function must follow some special rules:
        /// - Cannot use memory allocation services, directly or indirectly
        /// - Cannot depend on timer events, since those will be deactivated
        const SIGNAL_EXIT_BOOT_SERVICES = 0x0000_0201;

        /// The event will be notified when SetVirtualAddressMap is performed.
        /// This event type should not be combined with any other.
        const SIGNAL_VIRTUAL_ADDRESS_CHANGE = 0x6000_0202;
    }
}

/// Raw event notification function
type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: Option<NonNull<c_void>>);

/// Timer events manipulation.
#[derive(Debug)]
pub enum TimerTrigger {
    /// Cancel event's timer
    Cancel,
    /// The event is to be signaled periodically.
    /// Parameter is the period in 100ns units.
    /// Delay of 0 will be signalled on every timer tick.
    Periodic(u64),
    /// The event is to be signaled once in 100ns units.
    /// Parameter is the delay in 100ns units.
    /// Delay of 0 will be signalled on next timer tick.
    Relative(u64),
}

newtype_enum! {
/// Interface type of a protocol interface
///
/// Only has one variant when this was written (v2.10 of the UEFI spec)
pub enum InterfaceType: i32 => {
    /// Native interface
    NATIVE_INTERFACE    = 0,
}
}

/// Opaque pointer returned by [`BootServices::register_protocol_notify`] to be used
/// with [`BootServices::locate_handle`] via [`SearchType::ByRegisterNotify`].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ProtocolSearchKey(NonNull<c_void>);

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use crate::table::boot::{MemoryAttribute, MemoryMap, MemoryMapKey, MemoryType};

    use super::{MemoryDescriptor, MemoryMapIter};

    #[test]
    fn mem_map_sorting() {
        // Doesn't matter what type it is.
        const TY: MemoryType = MemoryType::RESERVED;

        const BASE: MemoryDescriptor = MemoryDescriptor {
            ty: TY,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            att: MemoryAttribute::empty(),
        };

        let mut buffer = [
            MemoryDescriptor {
                phys_start: 2000,
                ..BASE
            },
            MemoryDescriptor {
                phys_start: 3000,
                ..BASE
            },
            BASE,
            MemoryDescriptor {
                phys_start: 1000,
                ..BASE
            },
        ];

        let desc_count = buffer.len();

        let byte_buffer = {
            let size = desc_count * size_of::<MemoryDescriptor>();
            unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, size) }
        };

        let mut mem_map = MemoryMap {
            // Key doesn't matter
            key: MemoryMapKey(0),
            len: desc_count,
            buf: byte_buffer,
            entry_size: size_of::<MemoryDescriptor>(),
        };

        mem_map.sort();

        if !is_sorted(&mem_map.entries()) {
            panic!("mem_map is not sorted: {}", mem_map);
        }
    }

    // Added for debug purposes on test failure
    impl core::fmt::Display for MemoryMap<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            writeln!(f)?;
            for desc in self.entries() {
                writeln!(f, "{:?}", desc)?;
            }
            Ok(())
        }
    }

    fn is_sorted(iter: &MemoryMapIter) -> bool {
        let mut iter = iter.clone();
        let mut curr_start;

        if let Some(val) = iter.next() {
            curr_start = val.phys_start;
        } else {
            return true;
        }

        for desc in iter {
            if desc.phys_start <= curr_start {
                return false;
            }
            curr_start = desc.phys_start
        }
        true
    }
}
