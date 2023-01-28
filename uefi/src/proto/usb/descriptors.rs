// TODO
//#![allow(missing_docs)]

newtype_enum! {
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
    pub descriptor_type: DescriptorType,
    pub bcd_usb: u16,
    pub device_class: Class,
    pub device_sub_class: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub str_manufacturer: u8,
    pub str_product: u8,
    pub str_serial_number: u8,
    pub num_configurations: u8,
}

/// Information about a specific device configuration.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ConfigDescriptor {
    /// Size of this descriptor in bytes.
    pub length: u8,
    pub descriptor_type: DescriptorType,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration: u8,
    pub attributes: u8,
    pub max_power: u8,
}

/// Information about a specific interface within a configuration.
#[derive(Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct InterfaceDescriptor {
    /// Size of this descriptor in bytes.
    pub length: u8,
    pub descriptor_type: DescriptorType,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: Class,
    pub interface_sub_class: u8,
    pub interface_protocol: u8,
    pub interface: u8,
}

/// Information about an interface endpoint.
#[derive(Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct EndpointDescriptor {
    /// Size of this descriptor in bytes.
    pub length: u8,
    pub descriptor_type: DescriptorType,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}
