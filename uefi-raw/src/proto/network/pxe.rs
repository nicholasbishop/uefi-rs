//! PXE Base Code protocol.

use super::{IpAddress, MacAddress};
use crate::{guid, Char8, Guid, Identify, Status};
use bitflags::bitflags;
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use ptr_meta::Pointee;

/// PXE Base Code protocol
#[repr(C)]
#[allow(clippy::type_complexity)]
pub struct BaseCode {
    pub revision: u64,
    pub start: extern "efiapi" fn(this: &Self, use_ipv6: bool) -> Status,
    pub stop: extern "efiapi" fn(this: &Self) -> Status,
    pub dhcp: extern "efiapi" fn(this: &Self, sort_offers: bool) -> Status,
    pub discover: extern "efiapi" fn(
        this: &Self,
        ty: BootstrapType,
        layer: &mut u16,
        use_bis: bool,
        info: *const FfiDiscoverInfo,
    ) -> Status,
    pub mtftp: unsafe extern "efiapi" fn(
        this: &Self,
        operation: TftpOpcode,
        buffer: *mut c_void,
        overwrite: bool,
        buffer_size: &mut u64,
        block_size: Option<&usize>,
        server_ip: &IpAddress,
        filename: *const Char8,
        info: Option<&MtftpInfo>,
        dont_use_buffer: bool,
    ) -> Status,
    pub udp_write: unsafe extern "efiapi" fn(
        this: &Self,
        op_flags: UdpOpFlags,
        dest_ip: &IpAddress,
        dest_port: &u16,
        gateway_ip: Option<&IpAddress>,
        src_ip: Option<&IpAddress>,
        src_port: Option<&mut u16>,
        header_size: Option<&usize>,
        header_ptr: *const c_void,
        buffer_size: &usize,
        buffer_ptr: *const c_void,
    ) -> Status,
    pub udp_read: unsafe extern "efiapi" fn(
        this: &Self,
        op_flags: UdpOpFlags,
        dest_ip: Option<&mut IpAddress>,
        dest_port: Option<&mut u16>,
        src_ip: Option<&mut IpAddress>,
        src_port: Option<&mut u16>,
        header_size: Option<&usize>,
        header_ptr: *mut c_void,
        buffer_size: &mut usize,
        buffer_ptr: *mut c_void,
    ) -> Status,
    pub set_ip_filter: extern "efiapi" fn(this: &Self, new_filter: &IpFilter) -> Status,
    pub arp: extern "efiapi" fn(
        this: &Self,
        ip_addr: &IpAddress,
        mac_addr: Option<&mut MacAddress>,
    ) -> Status,
    pub set_parameters: extern "efiapi" fn(
        this: &Self,
        new_auto_arp: Option<&bool>,
        new_send_guid: Option<&bool>,
        new_ttl: Option<&u8>,
        new_tos: Option<&u8>,
        new_make_callback: Option<&bool>,
    ) -> Status,
    pub set_station_ip: extern "efiapi" fn(
        this: &Self,
        new_station_ip: Option<&IpAddress>,
        new_subnet_mask: Option<&IpAddress>,
    ) -> Status,
    pub set_packets: extern "efiapi" fn(
        this: &Self,
        new_dhcp_discover_valid: Option<&bool>,
        new_dhcp_ack_received: Option<&bool>,
        new_proxy_offer_received: Option<&bool>,
        new_pxe_discover_valid: Option<&bool>,
        new_pxe_reply_received: Option<&bool>,
        new_pxe_bis_reply_received: Option<&bool>,
        new_dhcp_discover: Option<&Packet>,
        new_dhcp_ack: Option<&Packet>,
        new_proxy_offer: Option<&Packet>,
        new_pxe_discover: Option<&Packet>,
        new_pxe_reply: Option<&Packet>,
        new_pxe_bis_reply: Option<&Packet>,
    ) -> Status,
    pub mode: *const Mode,
}

unsafe impl Identify for BaseCode {
    const GUID: Guid = guid!("03c4e603-ac28-11d3-9a2d-0090273fc14d");
}

/// A type of bootstrap to perform in [`BaseCode::discover`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_BOOT_` constants in the C API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
#[allow(missing_docs)]
pub enum BootstrapType {
    Bootstrap = 0,
    MsWinntRis = 1,
    IntelLcm = 2,
    DosUndi = 3,
    NecEsmpro = 4,
    IbmWsoD = 5,
    IbmLccm = 6,
    CaUnicenterTng = 7,
    HpOpenview = 8,
    Altiris9 = 9,
    Altiris10 = 10,
    Altiris11 = 11,
    // NOT_USED_12 = 12,
    RedhatInstall = 13,
    RedhatBoot = 14,
    Rembo = 15,
    Beoboot = 16,
    //
    // Values 17 through 32767 are reserved.
    // Values 32768 through 65279 are for vendor use.
    // Values 65280 through 65534 are reserved.
    //
    PxeTest = 65535,
}

opaque_type! {
    /// Opaque type that should be used to represent a pointer to a [`DiscoverInfo`] in
    /// foreign function interfaces. This type produces a thin pointer, unlike
    /// [`DiscoverInfo`].
    pub struct FfiDiscoverInfo;
}

/// This struct contains optional parameters for [`BaseCode::discover`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_DISCOVER_INFO` type in the C API.
#[repr(C)]
#[derive(Debug, Pointee)]
pub struct DiscoverInfo {
    pub use_m_cast: bool,
    pub use_b_cast: bool,
    pub use_u_cast: bool,
    pub must_use_list: bool,
    pub server_m_cast_ip: IpAddress,
    pub ip_cnt: u16,
    pub srv_list: [Server],
}

/// An entry in the Boot Server list
///
/// Corresponds to the `EFI_PXE_BASE_CODE_SRVLIST` type in the C API.
#[repr(C)]
#[derive(Debug)]
pub struct Server {
    /// The type of Boot Server reply
    pub ty: u16,
    pub accept_any_response: bool,
    pub reserved: u8,
    /// The IP address of the server
    pub ip_addr: IpAddress,
}

/// Corresponds to the `EFI_PXE_BASE_CODE_TFTP_OPCODE` type in the C API.
#[repr(C)]
pub enum TftpOpcode {
    TftpGetFileSize = 1,
    TftpReadFile,
    TftpWriteFile,
    TftpReadDirectory,
    MtftpGetFileSize,
    MtftpReadFile,
    MtftpReadDirectory,
}

/// MTFTP connection parameters
///
/// Corresponds to the `EFI_PXE_BASE_CODE_MTFTP_INFO` type in the C API.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MtftpInfo {
    /// File multicast IP address. This is the IP address to which the server
    /// will send the requested file.
    pub m_cast_ip: IpAddress,
    /// Client multicast listening port. This is the UDP port to which the
    /// server will send the requested file.
    pub c_port: u16,
    /// Server multicast listening port. This is the UDP port on which the
    /// server listens for multicast open requests and data acks.
    pub s_port: u16,
    /// The number of seconds a client should listen for an active multicast
    /// session before requesting a new multicast session.
    pub listen_timeout: u16,
    /// The number of seconds a client should wait for a packet from the server
    /// before retransmitting the previous open request or data ack packet.
    pub transmit_timeout: u16,
}

// No corresponding type in the UEFI spec, it just uses UINT16.
bitflags! {
    /// Flags for UDP read and write operations.
    #[repr(transparent)]
    pub struct UdpOpFlags: u16 {
        /// Receive a packet sent from any IP address in UDP read operations.
        const ANY_SRC_IP = 0x0001;
        /// Receive a packet sent from any UDP port in UDP read operations. If
        /// the source port is no specified in UDP write operations, the
        /// source port will be automatically selected.
        const ANY_SRC_PORT = 0x0002;
        /// Receive a packet sent to any IP address in UDP read operations.
        const ANY_DEST_IP = 0x0004;
        /// Receive a packet sent to any UDP port in UDP read operations.
        const ANY_DEST_PORT = 0x0008;
        /// The software filter is used in UDP read operations.
        const USE_FILTER = 0x0010;
        /// If required, a UDP write operation may be broken up across multiple packets.
        const MAY_FRAGMENT = 0x0020;
    }
}

/// IP receive filter settings
///
/// Corresponds to the `EFI_PXE_BASE_CODE_IP_FILTER` type in the C API.
#[repr(C)]
#[derive(Debug)]
pub struct IpFilter {
    /// A set of filters.
    pub filters: IpFilters,
    pub ip_cnt: u8,
    pub reserved: u16,
    pub ip_list: [IpAddress; 8],
}

bitflags! {
    /// IP receive filters.
    #[repr(transparent)]
    pub struct IpFilters: u8 {
        /// Enable the Station IP address.
        const STATION_IP = 0x01;
        /// Enable IPv4 broadcast addresses.
        const BROADCAST = 0x02;
        /// Enable all addresses.
        const PROMISCUOUS = 0x04;
        /// Enable all multicast addresses.
        const PROMISCUOUS_MULTICAST = 0x08;
    }
}

/// A network packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_PACKET` type in the C API.
#[repr(C)]
pub union Packet {
    raw: [u8; 1472],
    dhcpv4: DhcpV4Packet,
    dhcpv6: DhcpV6Packet,
}

impl Debug for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "<binary data>")
    }
}

impl AsRef<[u8; 1472]> for Packet {
    fn as_ref(&self) -> &[u8; 1472] {
        unsafe { &self.raw }
    }
}

impl AsRef<DhcpV4Packet> for Packet {
    fn as_ref(&self) -> &DhcpV4Packet {
        unsafe { &self.dhcpv4 }
    }
}

impl AsRef<DhcpV6Packet> for Packet {
    fn as_ref(&self) -> &DhcpV6Packet {
        unsafe { &self.dhcpv6 }
    }
}

/// A Dhcpv4 Packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_DHCPV4_PACKET` type in the C API.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DhcpV4Packet {
    /// Packet op code / message type.
    pub bootp_opcode: u8,
    /// Hardware address type.
    pub bootp_hw_type: u8,
    /// Hardware address length.
    pub bootp_hw_addr_len: u8,
    /// Client sets to zero, optionally used by gateways in cross-gateway booting.
    pub bootp_gate_hops: u8,
    pub bootp_ident: u32,
    pub bootp_seconds: u16,
    pub bootp_flags: u16,
    /// Client IP address, filled in by client in bootrequest if known.
    pub bootp_ci_addr: [u8; 4],
    /// 'your' (client) IP address; filled by server if client doesn't know its own address (`bootp_ci_addr` was 0).
    pub bootp_yi_addr: [u8; 4],
    /// Server IP address, returned in bootreply by server.
    pub bootp_si_addr: [u8; 4],
    /// Gateway IP address, used in optional cross-gateway booting.
    pub bootp_gi_addr: [u8; 4],
    /// Client hardware address, filled in by client.
    pub bootp_hw_addr: [u8; 16],
    /// Optional server host name, null terminated string.
    pub bootp_srv_name: [u8; 64],
    /// Boot file name, null terminated string, 'generic' name or null in
    /// bootrequest, fully qualified directory-path name in bootreply.
    pub bootp_boot_file: [u8; 128],
    pub dhcp_magik: u32,
    /// Optional vendor-specific area, e.g. could be hardware type/serial on request, or 'capability' / remote file system handle on reply.  This info may be set aside for use by a third phase bootstrap or kernel.
    pub dhcp_options: [u8; 56],
}

impl DhcpV4Packet {
    /// The expected value for [`Self::dhcp_magik`].
    pub const DHCP_MAGIK: u32 = 0x63825363;

    /// Transaction ID, a random number, used to match this boot request with the responses it generates.
    #[must_use]
    pub const fn bootp_ident(&self) -> u32 {
        u32::from_be(self.bootp_ident)
    }

    /// Filled in by client, seconds elapsed since client started trying to boot.
    #[must_use]
    pub const fn bootp_seconds(&self) -> u16 {
        u16::from_be(self.bootp_seconds)
    }

    /// The flags.
    #[must_use]
    pub const fn bootp_flags(&self) -> DhcpV4Flags {
        DhcpV4Flags::from_bits_truncate(u16::from_be(self.bootp_flags))
    }

    /// A magic cookie, should be [`Self::DHCP_MAGIK`].
    #[must_use]
    pub const fn dhcp_magik(&self) -> u32 {
        u32::from_be(self.dhcp_magik)
    }
}

bitflags! {
    /// Represents the 'flags' field for a [`DhcpV4Packet`].
    #[repr(transparent)]
    pub struct DhcpV4Flags: u16 {
        /// Should be set when the client cannot receive unicast IP datagrams
        /// until its protocol software has been configured with an IP address.
        const BROADCAST = 1;
    }
}

/// A Dhcpv6 Packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_DHCPV6_PACKET` type in the C API.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DhcpV6Packet {
    /// The message type.
    pub message_type: u8,
    pub transaction_id: [u8; 3],
    /// A byte array containing dhcp options.
    pub dhcp_options: [u8; 1024],
}

impl DhcpV6Packet {
    /// The transaction id.
    #[must_use]
    pub fn transaction_id(&self) -> u32 {
        u32::from(self.transaction_id[0]) << 16
            | u32::from(self.transaction_id[1]) << 8
            | u32::from(self.transaction_id[2])
    }
}

/// The data values in this structure are read-only and are updated by the
/// [`BaseCode`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_MODE` type in the C API.
#[repr(C)]
#[derive(Debug)]
pub struct Mode {
    /// `true` if this device has been started by calling [`BaseCode::start`].
    /// This field is set to `true` by [`BaseCode::start`] and to `false` by
    /// the [`BaseCode::stop`] function.
    pub started: bool,
    /// `true` if the UNDI protocol supports IPv6
    pub ipv6_available: bool,
    /// `true` if this PXE Base Code Protocol implementation supports IPv6.
    pub ipv6_supported: bool,
    /// `true` if this device is currently using IPv6. This field is set by
    /// [`BaseCode::start`].
    pub using_ipv6: bool,
    /// `true` if this PXE Base Code implementation supports Boot Integrity
    /// Services (BIS). This field is set by [`BaseCode::start`].
    pub bis_supported: bool,
    /// `true` if this device and the platform support Boot Integrity Services
    /// (BIS). This field is set by [`BaseCode::start`].
    pub bis_detected: bool,
    /// `true` for automatic ARP packet generation, `false` otherwise. This
    /// field is initialized to `true` by [`BaseCode::start`] and can be
    /// modified with [`BaseCode::set_parameters`].
    pub auto_arp: bool,
    /// This field is used to change the Client Hardware Address (chaddr) field
    /// in the DHCP and Discovery packets. Set to `true` to send the SystemGuid
    /// (if one is available). Set to `false` to send the client NIC MAC
    /// address. This field is initialized to `false` by [`BaseCode::start`]
    /// and can be modified with [`BaseCode::set_parameters`].
    pub send_guid: bool,
    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully. When `true`,
    /// [`Self::dhcp_discover`] is valid. This field can also be changed by
    /// [`BaseCode::set_packets`].
    pub dhcp_discover_valid: bool,
    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully. When `true`,
    /// [`Self::dhcp_ack`] is valid. This field can also be changed by
    /// [`BaseCode::set_packets`].
    pub dhcp_ack_received: bool,
    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully and a proxy DHCP
    /// offer packet was received. When `true`, [`Self::proxy_offer`] is valid.
    /// This field can also be changed by [`BaseCode::set_packets`].
    pub proxy_offer_received: bool,
    /// When `true`, [`Self::pxe_discover`] is valid. This field is set to
    /// `false` by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set
    /// to `true` or `false` by [`BaseCode::discover`] and
    /// [`BaseCode::set_packets`].
    pub pxe_discover_valid: bool,
    /// When `true`, [`Self::pxe_reply`] is valid. This field is set to `false`
    /// by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set to `true`
    /// or `false` by [`BaseCode::discover`] and [`BaseCode::set_packets`].
    pub pxe_reply_received: bool,
    /// When `true`, [`Self::pxe_bis_reply`] is valid. This field is set to
    /// `false` by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set
    /// to `true` or `false` by the [`BaseCode::discover`] and
    /// [`BaseCode::set_packets`].
    pub pxe_bis_reply_received: bool,
    /// Indicates whether [`Self::icmp_error`] has been updated. This field is
    /// reset to `false` by [`BaseCode::start`], [`BaseCode::dhcp`],
    /// [`BaseCode::discover`],[`BaseCode::udp_read`], [`BaseCode::udp_write`],
    /// [`BaseCode::arp`] and any of the TFTP/MTFTP operations. If an ICMP
    /// error is received, this field will be set to `true` after
    /// [`Self::icmp_error`] is updated.
    pub icmp_error_received: bool,
    /// Indicates whether [`Self::tftp_error`] has been updated. This field is
    /// reset to `false` by [`BaseCode::start`] and any of the TFTP/MTFTP
    /// operations. If a TFTP error is received, this field will be set to
    /// `true` after [`Self::tftp_error`] is updated.
    pub tftp_error_received: bool,
    /// When `false`, callbacks will not be made. When `true`, make callbacks
    /// to the PXE Base Code Callback Protocol. This field is reset to `false`
    /// by [`BaseCode::start`] if the PXE Base Code Callback Protocol is not
    /// available. It is reset to `true` by [`BaseCode::start`] if the PXE Base
    /// Code Callback Protocol is available.
    pub make_callbacks: bool,
    /// The "time to live" field of the IP header. This field is initialized to
    /// `16` by [`BaseCode::start`] and can be modified by
    /// [`BaseCode::set_parameters`].
    pub ttl: u8,
    /// The type of service field of the IP header. This field is initialized
    /// to `0` by [`BaseCode::start`], and can be modified with
    /// [`BaseCode::set_parameters`].
    pub tos: u8,
    /// The device’s current IP address. This field is initialized to a zero
    /// address by Start(). This field is set when [`BaseCode::dhcp`] completes
    /// successfully. This field can also be set by
    /// [`BaseCode::set_station_ip`]. This field must be set to a valid IP
    /// address by either [`BaseCode::dhcp`] or [`BaseCode::set_station_ip`]
    /// before [`BaseCode::discover`], [`BaseCode::udp_read`],
    /// [`BaseCode::udp_write`], [`BaseCode::arp`] and any of the TFTP/MTFTP
    /// operations are called.
    pub station_ip: IpAddress,
    /// The device's current subnet mask. This field is initialized to a zero
    /// address by [`BaseCode::start`]. This field is set when
    /// [`BaseCode::dhcp`] completes successfully. This field can also be set
    /// by [`BaseCode::set_station_ip`]. This field must be set to a valid
    /// subnet mask by either [`BaseCode::dhcp`] or
    /// [`BaseCode::set_station_ip`] before [`BaseCode::discover`],
    /// [`BaseCode::udp_read`], [`BaseCode::udp_write`],
    /// [`BaseCode::arp`] or any of the TFTP/MTFTP operations are called.
    pub subnet_mask: IpAddress,
    /// Cached DHCP Discover packet. This field is zero-filled by the
    /// [`BaseCode::start`] function, and is set when [`BaseCode::dhcp`]
    /// completes successfully. The contents of this field can replaced by
    /// [`BaseCode::set_packets`].
    pub dhcp_discover: Packet,
    /// Cached DHCP Ack packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::dhcp`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    pub dhcp_ack: Packet,
    /// Cached Proxy Offer packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::dhcp`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    pub proxy_offer: Packet,
    /// Cached PXE Discover packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    pub pxe_discover: Packet,
    /// Cached PXE Reply packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. The contents of this field can be replaced by the
    /// [`BaseCode::set_packets`] function.
    pub pxe_reply: Packet,
    /// Cached PXE BIS Reply packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. This field can be replaced by [`BaseCode::set_packets`].
    pub pxe_bis_reply: Packet,
    /// The current IP receive filter settings. The receive filter is disabled
    /// and the number of IP receive filters is set to zero by
    /// [`BaseCode::start`], and is set by [`BaseCode::set_ip_filter`].
    pub ip_filter: IpFilter,
    /// The number of valid entries in the ARP cache. This field is reset to
    /// zero by [`BaseCode::start`].
    pub arp_cache_entries: u32,
    /// Array of cached ARP entries.
    pub arp_cache: [ArpEntry; 8],
    /// The number of valid entries in the current route table. This field is
    /// reset to zero by [`BaseCode::start`].
    pub route_table_entries: u32,
    /// Array of route table entries.
    pub route_table: [RouteEntry; 8],
    /// ICMP error packet. This field is updated when an ICMP error is received
    /// and is undefined until the first ICMP error is received. This field is
    /// zero-filled by [`BaseCode::start`].
    pub icmp_error: IcmpError,
    /// TFTP error packet. This field is updated when a TFTP error is received
    /// and is undefined until the first TFTP error is received. This field is
    /// zero-filled by the [`BaseCode::start`] function.
    pub tftp_error: TftpError,
}

/// An entry for the ARP cache found in [`Mode::arp_cache`]
///
/// Corresponds to the `EFI_PXE_BASE_CODE_ARP_ENTRY` type in the C API.
#[repr(C)]
#[derive(Debug)]
pub struct ArpEntry {
    /// The IP address.
    pub ip_addr: IpAddress,
    /// The mac address of the device that is addressed by [`Self::ip_addr`].
    pub mac_addr: MacAddress,
}

/// An entry for the route table found in [`Mode::route_table`]
///
/// Corresponds to the `EFI_PXE_BASE_CODE_ROUTE_ENTRY` type in the C API.
#[repr(C)]
#[allow(missing_docs)]
#[derive(Debug)]
pub struct RouteEntry {
    pub ip_addr: IpAddress,
    pub subnet_mask: IpAddress,
    pub gw_addr: IpAddress,
}

/// An ICMP error packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_ICMP_ERROR` type in the C API.
#[repr(C)]
#[allow(missing_docs)]
#[derive(Debug)]
pub struct IcmpError {
    pub ty: u8,
    pub code: u8,
    pub checksum: u16,
    pub u: IcmpErrorUnion,
    pub data: [u8; 494],
}

/// Corresponds to the anonymous union inside
/// `EFI_PXE_BASE_CODE_ICMP_ERROR` in the C API.
#[repr(C)]
#[allow(missing_docs)]
pub union IcmpErrorUnion {
    pub reserved: u32,
    pub mtu: u32,
    pub pointer: u32,
    pub echo: IcmpErrorEcho,
}

impl Debug for IcmpErrorUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "<binary data>")
    }
}

/// Corresponds to the `Echo` field in the anonymous union inside
/// `EFI_PXE_BASE_CODE_ICMP_ERROR` in the C API.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
pub struct IcmpErrorEcho {
    pub identifier: u16,
    pub sequence: u16,
}

/// A TFTP error packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_TFTP_ERROR` type in the C API.
#[repr(C)]
#[allow(missing_docs)]
#[derive(Debug)]
pub struct TftpError {
    pub error_code: u8,
    pub error_string: [u8; 127],
}
