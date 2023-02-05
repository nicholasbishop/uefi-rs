// TODO
//#![allow(missing_docs)]

// TODO link to
// https://www.usb.org/document-library/usb-20-specification in the docstring

use bitflags::bitflags;

newtype_enum! {
    /// USB device class code.
    ///
    /// See <https://www.usb.org/defined-class-codes>.
    pub enum Class: u8 => #[allow(missing_docs)] {
        USE_INTERFACE_DESCRIPTORS = 0x00,
        AUDIO = 0x01,
        COMMUNICATIONS = 0x02,
        HID = 0x03,
        PHYSICAL = 0x05,
        IMAGE = 0x06,
        PRINTER = 0x07,
        MASS_STORAGE = 0x08,
        HUB = 0x09,
        CDC_DATA = 0x0a,
        SMART_CARD = 0x0b,
        CONTENT_SECURITY = 0x0d,
        VIDEO = 0x0e,
        PERSONAL_HEALTHCARE = 0x0f,
        AUDIO_VIDEO = 0x10,
        BILLBOARD = 0x11,
        USB_TYPE_C_BRIDGE = 0x12,
        I3C = 0x3c,
        DIAGNOSTICS = 0xdc,
        WIRELESS_CONTROLLER = 0xe0,
        MISCELLANEOUS = 0xef,
        APPLICATION_SPECIFIC = 0xfe,
        VENDOR_SPECIFIC = 0xff,
    }
}

impl Default for Class {
    fn default() -> Self {
        Class(0)
    }
}

newtype_enum! {
    /// Type of USB descriptor.
    pub enum DescriptorType: u8 => #[allow(missing_docs)] {
        DEVICE = 0x01,
        CONFIGURATION = 0x02,
        STRING = 0x03,
        INTERFACE = 0x04,
        ENDPOINT = 0x05,
        DEVICE_QUALIFIER = 0x06,
        OTHER_SPEED_CONFIGURATION = 0x07,
        INTERFACE_POWER = 0x08,
    }
}

impl Default for DescriptorType {
    fn default() -> Self {
        DescriptorType(0)
    }
}

/// General information about a USB device.
#[derive(Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct DeviceDescriptor {
    /// Size of this descriptor in bytes.
    pub length: u8,

    /// Type of descriptor ([`DescriptorType::DEVICE`]).
    pub descriptor_type: DescriptorType,

    /// USB Specification release number in binary-coded decimal. For example,
    /// 0x0210 would indicate release 2.10.
    pub bcd_usb: u16,

    /// Device class code.
    ///
    /// See <https://www.usb.org/defined-class-codes>.
    ///
    /// If this is set to [`Class::USE_INTERFACE_DESCRIPTORS`], each interface
    /// within a configuration specifies its own class information, and the
    /// various interfaces operate independently.
    pub device_class: Class,

    /// Device subclass code.
    ///
    /// The meaning of this field depends on the class. See
    /// <https://www.usb.org/defined-class-codes> for details.
    pub device_sub_class: u8,

    /// Device protocol code.
    ///
    /// The meaning of this field depends on the class and subclass. See
    /// <https://www.usb.org/defined-class-codes> for details.
    pub device_protocol: u8,

    /// Maximum packet size for endpoint zero (only 8, 16, 32, or 64 are valid).
    pub max_packet_size0: u8,

    /// Vendor ID.
    pub id_vendor: u16,

    /// Product ID.
    pub id_product: u16,

    /// Device release number in binary-coded decimal.
    pub bcd_device: u16,

    /// Index of the manufacturer string descriptor.
    pub str_manufacturer: u8,

    /// Index of the product string descriptor.
    pub str_product: u8,

    /// Index of the serial number string descriptor.
    pub str_serial_number: u8,

    /// Number of possible configurations.
    pub num_configurations: u8,
}

bitflags! {
    /// Attributes for a [`ConfigDescriptor`].
    #[derive(Default)]
    pub struct ConfigAttributes: u8 {
        /// Reserved, set to 0.
        const D0_RESERVED = 0b0000_0001;

        /// Reserved, set to 0.
        const D1_RESERVED = 0b0000_0010;

        /// Reserved, set to 0.
        const D2_RESERVED = 0b0000_0100;

        /// Reserved, set to 0.
        const D3_RESERVED = 0b0000_1000;

        /// Reserved, set to 0.
        const D4_RESERVED = 0b0001_0000;

        /// Device supports remote wakeup.
        const D5_REMOTE_WAKEUP = 0b0010_0000;

        /// Device uses power from a local source.
        const D6_SELF_POWERED = 0b0100_0000;

        /// Reserved, set to 1.
        const D7_RESERVED = 0b1000_0000;
    }
}

/// Information about a specific device configuration.
#[derive(Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct ConfigDescriptor {
    /// Size of this descriptor in bytes.
    pub length: u8,

    /// Type of descriptor ([`DescriptorType::CONFIGURATION`]).
    pub descriptor_type: DescriptorType,

    /// Total length of data returned for this configuration.
    pub total_length: u16,

    /// Number of interfaces supported by this configuration.
    pub num_interfaces: u8,

    /// Value to use when identifying this configuration.
    pub configuration_value: u8,

    /// Index of the configuration string descriptor.
    pub configuration: u8,

    /// Configuration attributes.
    pub attributes: ConfigAttributes,

    /// Maximum power consumption of the device in this configuration when fully
    /// operational, expressed in 2 mA units.
    pub max_power: u8,
}

/// Information about a specific interface within a configuration.
#[derive(Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct InterfaceDescriptor {
    /// Size of this descriptor in bytes.
    pub length: u8,

    /// Type of descriptor ([`DescriptorType::INTERFACE`]).
    pub descriptor_type: DescriptorType,

    /// Zero-based index of this interface.
    pub interface_number: u8,

    /// Value used to select the alternate setting for this interface.
    pub alternate_setting: u8,

    /// Number of endpoints used by this interface, excluding endpoint zero.
    pub num_endpoints: u8,

    /// Class code.
    ///
    /// See <https://www.usb.org/defined-class-codes>.
    ///
    /// The value of zero is reserved.
    pub interface_class: Class,

    /// Subclass code.
    ///
    /// The meaning of this field depends on the class. See
    /// <https://www.usb.org/defined-class-codes> for details.
    pub interface_sub_class: u8,

    /// Protocol code.
    ///
    /// The meaning of this field depends on the class and subclass. See
    /// <https://www.usb.org/defined-class-codes> for details.
    pub interface_protocol: u8,

    /// Index of the interface string descriptor.
    pub interface: u8,
}

/// Information about an interface endpoint.
#[derive(Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct EndpointDescriptor {
    /// Size of this descriptor in bytes.
    pub length: u8,

    /// Type of descriptor ([`DescriptorType::ENDPOINT`]).
    pub descriptor_type: DescriptorType,

    /// Address of this endpoint.
    ///
    /// * Bits `3..0`: endpoint number.
    /// * Bits `6..4`: reserved, set to zero.
    /// * Bit `7`: Direction, ignored for control endpoints. `0` means OUT, `1` means IN.
    pub endpoint_address: u8,

    /// Endpoint attributes.
    ///
    /// * Bits `1..0` are the transfer type.
    /// * For isochronous endpoints only:
    ///   * Bits `3..2` are the synchronization type.
    ///   * Bits `5..4` are the synchronization type.
    pub attributes: u8,

    /// Maximum packet size this endpoint is capable of sending or receiving
    /// when this configuration is selected.
    ///
    /// The actual packet size (in bytes) is in bits `10..0`. Bits `12..11`
    /// specify additional flags in high-speed isochronous and interrupt
    /// endpoints. Bits `15..13` are reserved.
    pub max_packet_size: u16,

    /// Interval for polling endpoint for data transfers. Expressed in frames
    /// or microframes depending on the device operating speed.
    pub interval: u8,
}
