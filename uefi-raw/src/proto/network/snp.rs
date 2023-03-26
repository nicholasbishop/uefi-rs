//! Simple Network Protocol
//!
//! Provides a packet level interface to a network adapter.
//! Once the adapter is initialized, the protocol provides services that allows
//! packets to be transmitted and received.
//!
//! No interface function must be called until `SimpleNetwork.start` is successfully
//! called first.

use super::{IpAddress, MacAddress};
use crate::{guid, Event, Guid, Identify, Status};
use bitflags::bitflags;
use core::ffi::c_void;
use core::ptr::NonNull;

/// The Simple Network Protocol
#[repr(C)]
pub struct SimpleNetwork {
    pub revision: u64,
    pub start: extern "efiapi" fn(this: &Self) -> Status,
    pub stop: extern "efiapi" fn(this: &Self) -> Status,
    pub initialize: extern "efiapi" fn(
        this: &Self,
        extra_recv_buffer_size: usize,
        extra_transmit_buffer_size: usize,
    ) -> Status,
    pub reset: extern "efiapi" fn(this: &Self, extended_verification: bool) -> Status,
    pub shutdown: extern "efiapi" fn(this: &Self) -> Status,
    pub receive_filters: extern "efiapi" fn(
        this: &Self,
        enable: u32,
        disable: u32,
        reset_mcast_filter: bool,
        mcast_filter_count: usize,
        mcast_filter: Option<NonNull<MacAddress>>,
    ) -> Status,
    pub station_address:
        extern "efiapi" fn(this: &Self, reset: bool, new: Option<&MacAddress>) -> Status,
    pub statistics: extern "efiapi" fn(
        this: &Self,
        reset: bool,
        stats_size: Option<&mut usize>,
        stats_table: Option<&mut NetworkStats>,
    ) -> Status,
    pub mcast_ip_to_mac:
        extern "efiapi" fn(this: &Self, ipv6: bool, ip: &IpAddress, mac: &mut MacAddress) -> Status,
    pub nv_data: extern "efiapi" fn(
        this: &Self,
        read_write: bool,
        offset: usize,
        buffer_size: usize,
        buffer: *mut c_void,
    ) -> Status,
    pub get_status: extern "efiapi" fn(
        this: &Self,
        interrupt_status: Option<&mut InterruptStatus>,
        tx_buf: Option<&mut *mut c_void>,
    ) -> Status,
    pub transmit: extern "efiapi" fn(
        this: &Self,
        header_size: usize,
        buffer_size: usize,
        buffer: *const c_void,
        src_addr: Option<&MacAddress>,
        dest_addr: Option<&MacAddress>,
        protocol: Option<&u16>,
    ) -> Status,
    pub receive: extern "efiapi" fn(
        this: &Self,
        header_size: Option<&mut usize>,
        buffer_size: &mut usize,
        buffer: *mut c_void,
        src_addr: Option<&mut MacAddress>,
        dest_addr: Option<&mut MacAddress>,
        protocol: Option<&mut u16>,
    ) -> Status,
    // On QEMU, this event seems to never fire.
    pub wait_for_packet: Event,
    pub mode: *const NetworkMode,
}

unsafe impl Identify for SimpleNetwork {
    const GUID: Guid = guid!("a19832b9-ac25-11d3-9a2d-0090273fc14d");
}

bitflags! {
    /// Flags to pass to receive_filters to enable/disable reception of some kinds of packets.
    #[repr(transparent)]
    pub struct ReceiveFlags : u32 {
        /// Receive unicast packets.
        const UNICAST = 0x01;
        /// Receive multicast packets.
        const MULTICAST = 0x02;
        /// Receive broadcast packets.
        const BROADCAST = 0x04;
        /// Receive packets in promiscuous mode.
        const PROMISCUOUS = 0x08;
        /// Receive packets in promiscuous multicast mode.
        const PROMISCUOUS_MULTICAST = 0x10;
    }
}

bitflags! {
    /// Flags returned by get_interrupt_status to indicate which interrupts have fired on the
    /// interface since the last call.
    #[repr(transparent)]
    pub struct InterruptStatus : u32 {
        /// Packet received.
        const RECEIVE = 0x01;
        /// Packet transmitted.
        const TRANSMIT = 0x02;
        /// Command interrupt fired.
        const COMMAND = 0x04;
        /// Software interrupt fired.
        const SOFTWARE = 0x08;
    }
}

/// Network Statistics
///
/// The description of statistics on the network with the SNP's `statistics` function
/// is returned in this structure
///
/// Any of these statistics may or may not be available on the device. So, all the
/// retriever functions of the statistics return `None` when a statistic is not supported
#[repr(C)]
#[derive(Default, Debug)]
pub struct NetworkStats {
    pub rx_total_frames: u64,
    pub rx_good_frames: u64,
    pub rx_undersize_frames: u64,
    pub rx_oversize_frames: u64,
    pub rx_dropped_frames: u64,
    pub rx_unicast_frames: u64,
    pub rx_broadcast_frames: u64,
    pub rx_multicast_frames: u64,
    pub rx_crc_error_frames: u64,
    pub rx_total_bytes: u64,
    pub tx_total_frames: u64,
    pub tx_good_frames: u64,
    pub tx_undersize_frames: u64,
    pub tx_oversize_frames: u64,
    pub tx_dropped_frames: u64,
    pub tx_unicast_frames: u64,
    pub tx_broadcast_frames: u64,
    pub tx_multicast_frames: u64,
    pub tx_crc_error_frames: u64,
    pub tx_total_bytes: u64,
    pub collisions: u64,
    pub unsupported_protocol: u64,
    pub rx_duplicated_frames: u64,
    pub rx_decrypt_error_frames: u64,
    pub tx_error_frames: u64,
    pub tx_retry_frames: u64,
}

/// The Simple Network Mode
#[repr(C)]
#[derive(Debug)]
pub struct NetworkMode {
    /// Reports the current state of the network interface
    pub state: NetworkState,
    /// The size of the network interface's hardware address in bytes
    pub hw_address_size: u32,
    /// The size of the network interface's media header in bytes
    pub media_header_size: u32,
    /// The maximum size of the packets supported by the network interface in bytes
    pub max_packet_size: u32,
    /// The size of the NVRAM device attached to the network interface in bytes
    pub nv_ram_size: u32,
    /// The size that must be used for all NVRAM reads and writes
    pub nv_ram_access_size: u32,
    /// The multicast receive filter settings supported by the network interface
    pub receive_filter_mask: u32,
    /// The current multicast receive filter settings
    pub receive_filter_setting: u32,
    /// The maximum number of multicast address receive filters supported by the driver
    pub max_mcast_filter_count: u32,
    /// The current number of multicast address receive filters
    pub mcast_filter_count: u32,
    /// The array containing the addresses of the current multicast address receive filters
    pub mcast_filter: [MacAddress; 16],
    /// The current hardware MAC address for the network interface
    pub current_address: MacAddress,
    /// The current hardware MAC address for broadcast packets
    pub broadcast_address: MacAddress,
    /// The permanent hardware MAC address for the network interface
    pub permanent_address: MacAddress,
    /// The interface type of the network interface
    pub if_type: u8,
    /// Tells if the MAC address can be changed
    pub mac_address_changeable: bool,
    /// Tells if the network interface can transmit more than one packet at a time
    pub multiple_tx_supported: bool,
    /// Tells if the presence of the media can be determined
    pub media_present_supported: bool,
    /// Tells if media are connected to the network interface
    pub media_present: bool,
}

newtype_enum! {
    /// The state of a network interface.
    pub enum NetworkState: u32 => {
        /// The interface has been stopped
        STOPPED = 0,
        /// The interface has been started
        STARTED = 1,
        /// The interface has been initialized
        INITIALIZED = 2,
        /// No state can have a number higher than this
        MAX_STATE = 4,
    }
}
