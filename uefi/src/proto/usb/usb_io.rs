//! TODO

// TODO
#![allow(missing_docs)]

use bitflags::bitflags;
use core::ffi::c_void;
use uefi::proto::unsafe_protocol;
use uefi::{Char16, Result, Status};

// TODO: pub?
pub struct UsbDeviceRequest {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

newtype_enum! {
    pub enum UsbDataDirection: u8 => {
        IN = 0,
        OUT = 1,
        NO_DATA = 2,
    }
}

bitflags! {
    pub struct UsbTransferError: u32 {
        const NOTEXECUTE = 0x0001;
        const STALL = 0x0002;
        const BUFFER = 0x0004;
        const BABBLE = 0x0008;
        const NAK = 0x0010;
        const CRC = 0x0020;
        const TIMEOUT = 0x0040;
        const BITSTUFF = 0x0080;
        const SYSTEM = 0x0100;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct UsbDeviceEndpoint(u8);

impl UsbDeviceEndpoint {
    pub fn new(device_endpoint: u8) -> Option<Self> {
        if (0x01..=0x0f).contains(&device_endpoint) || (0x81..=0x8f).contains(&device_endpoint) {
            Some(Self(device_endpoint))
        } else {
            None
        }
    }

    pub fn direction(self) -> UsbDataDirection {
        if self.0 & 0b1000_0000 == 0 {
            UsbDataDirection::OUT
        } else {
            UsbDataDirection::IN
        }
    }
}

// TODO: pub?
type AsyncUsbTransferCallback =
    fn(data: *mut c_void, data_length: usize, context: *mut c_void, status: u32);

#[derive(Debug, Default)]
#[repr(C)]
pub struct UsbDeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub bcd_usb: u16,
    pub device_class: u8,
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

#[derive(Debug, Default)]
pub struct UsbConfigDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration: u8,
    pub attributes: u8,
    pub max_power: u8,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct UsbInterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_sub_class: u8,
    pub interface_protocol: u8,
    pub interface: u8,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct UsbEndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

#[unsafe_protocol("2b2f68d6-0cd2-44cf-8e8b-bba20b1b5b75")]
#[repr(C)]
pub struct UsbIo {
    usb_control_transfer: unsafe extern "efiapi" fn(
        this: *const Self,
        request: *const UsbDeviceRequest,
        direction: UsbDataDirection,
        timeout: u32,
        data: *mut c_void,
        data_length: usize,
        status: *mut u32,
    ) -> Status,
    usb_bulk_transfer: unsafe extern "efiapi" fn(
        this: *const Self,
        device_endpoint: u8,
        data: *mut c_void,
        data_length: *mut usize,
        timeout: usize,
        status: *mut u32,
    ) -> Status,
    usb_async_interrupt_transfer: unsafe extern "efiapi" fn(
        this: *const Self,
        device_endpoint: u8,
        is_new_transfer: bool,
        polling_interval: usize,
        data_length: usize,
        interrupt_callBack: AsyncUsbTransferCallback,
        context: *mut c_void,
    ) -> Status,
    usb_sync_interrupt_transfer: unsafe extern "efiapi" fn(
        this: *const Self,
        device_endpoint: u8,
        data: *mut c_void,
        data_length: *mut usize,
        timeout: usize,
        status: *mut u32,
    ) -> Status,
    usb_isochronous_transfer: unsafe extern "efiapi" fn(
        this: *const Self,
        device_endpoint: u8,
        data: *mut c_void,
        data_length: usize,
        status: *mut u32,
    ) -> Status,
    usb_async_isochronous_transfer: unsafe extern "efiapi" fn(
        this: *const Self,
        device_endpoint: u8,
        data: *mut c_void,
        data_length: usize,
        isochronous_callback: AsyncUsbTransferCallback,
        context: *mut c_void,
    ) -> Status,
    usb_get_device_descriptor: unsafe extern "efiapi" fn(
        this: *const Self,
        device_descriptor: *mut UsbDeviceDescriptor,
    ) -> Status,
    usb_get_config_descriptor: unsafe extern "efiapi" fn(
        this: *const Self,
        config_descriptor: *mut UsbConfigDescriptor,
    ) -> Status,
    usb_get_interface_descriptor: unsafe extern "efiapi" fn(
        this: *const Self,
        interface_descriptor: *mut UsbInterfaceDescriptor,
    ) -> Status,
    usb_get_endpoint_descriptor: unsafe extern "efiapi" fn(
        this: *const Self,
        endpoint_descriptor: *mut UsbEndpointDescriptor,
    ) -> Status,
    usb_get_string_descriptor: unsafe extern "efiapi" fn(
        this: *const Self,
        lang_id: u16,
        string_id: u8,
        string: *mut *mut Char16,
    ) -> Status,
    usb_get_supported_languages:
        unsafe extern "efiapi" fn(this: *const Self, lang_id_table: *mut *mut u16) -> Status,
    usb_port_reset: unsafe extern "efiapi" fn(this: *const Self) -> Status,
}

impl UsbIo {
    // TODO: any way to combine some of these transfer functions with an input enum?

    pub fn usb_control_transfer(
        &self,
        request: &UsbDeviceRequest,
        direction: UsbDataDirection,
        timeout: u32,
        data: &mut [u8],
    ) -> Result<(), UsbTransferError> {
        let mut transfer_error = 0;
        unsafe {
            (self.usb_control_transfer)(
                self,
                request,
                direction,
                timeout,
                data.as_mut_ptr().cast(),
                data.len(),
                &mut transfer_error,
            )
            .into_with_err(|_| UsbTransferError::from_bits_unchecked(transfer_error))
        }
    }

    pub fn usb_bulk_transfer(
        &mut self,
        device_endpoint: UsbDeviceEndpoint,
        data: &mut [u8],
        data_length: *mut usize,
        timeout: usize,
        status: *mut u32,
    ) -> Result {
        todo!()
    }

    pub fn usb_async_interrupt_transfer(
        &mut self,
        device_endpoint: UsbDeviceEndpoint,
        is_new_transfer: bool,
        polling_interval: usize,
        data_length: usize,
        interrupt_callBack: AsyncUsbTransferCallback,
        context: *mut c_void,
    ) -> Result {
        todo!()
    }

    pub fn usb_sync_interrupt_transfer(
        &mut self,
        device_endpoint: UsbDeviceEndpoint,
        data: *mut c_void,
        data_length: *mut usize,
        timeout: usize,
        status: *mut u32,
    ) -> Result {
        todo!()
    }

    pub fn usb_isochronous_transfer(
        &mut self,
        device_endpoint: UsbDeviceEndpoint,
        data: *mut c_void,
        data_length: usize,
        status: *mut u32,
    ) -> Result {
        todo!()
    }

    pub fn usb_async_isochronous_transfer(
        &mut self,
        device_endpoint: UsbDeviceEndpoint,
        data: *mut c_void,
        data_length: usize,
        isochronous_callback: AsyncUsbTransferCallback,
        context: *mut c_void,
    ) -> Result {
        todo!()
    }

    pub fn usb_get_device_descriptor(
        &mut self,
        device_descriptor: *mut UsbDeviceDescriptor,
    ) -> Result {
        todo!()
    }

    pub fn usb_get_config_descriptor(
        &mut self,
        config_descriptor: *mut UsbConfigDescriptor,
    ) -> Result {
        todo!()
    }

    pub fn usb_get_interface_descriptor(
        &mut self,
        interface_descriptor: *mut UsbInterfaceDescriptor,
    ) -> Result {
        todo!()
    }

    pub fn usb_get_endpoint_descriptor(
        &mut self,
        endpoint_descriptor: *mut UsbEndpointDescriptor,
    ) -> Result {
        todo!()
    }

    pub fn usb_get_string_descriptor(
        &mut self,
        lang_id: u16,
        string_id: u8,
        string: *mut *mut Char16,
    ) -> Result {
        todo!()
    }

    pub fn usb_get_supported_languages(&mut self, lang_id_table: *mut *mut u16) -> Result {
        todo!()
    }

    pub fn usb_port_reset(&mut self) -> Result {
        todo!()
    }
}
