//! TODO

// TODO
#![allow(missing_docs)]

use core::{ptr, slice};
use uefi::data_types::PoolString;
use uefi::proto::unsafe_protocol;
use uefi::table::boot::BootServices;
use uefi::{Char16, Result, Status};

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
    // TODO: fill in the rest of these function pointers:
    usb_control_transfer: unsafe extern "efiapi" fn() -> Status,
    usb_bulk_transfer: unsafe extern "efiapi" fn() -> Status,
    usb_async_interrupt_transfer: unsafe extern "efiapi" fn() -> Status,
    usb_sync_interrupt_transfer: unsafe extern "efiapi" fn() -> Status,
    usb_isochronous_transfer: unsafe extern "efiapi" fn() -> Status,
    usb_async_isochronous_transfer: unsafe extern "efiapi" fn() -> Status,

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
        endpoint_index: u8,
        endpoint_descriptor: *mut UsbEndpointDescriptor,
    ) -> Status,
    usb_get_string_descriptor: unsafe extern "efiapi" fn(
        this: *const Self,
        lang_id: u16,
        string_id: u8,
        string: *mut *const Char16,
    ) -> Status,
    usb_get_supported_languages: unsafe extern "efiapi" fn(
        this: *const Self,
        lang_id_table: *mut *const u16,
        table_size: *mut u16,
    ) -> Status,
    usb_port_reset: unsafe extern "efiapi" fn(this: *const Self) -> Status,
}

impl UsbIo {
    pub fn get_device_descriptor(&self) -> Result<UsbDeviceDescriptor> {
        let mut device_descriptor = UsbDeviceDescriptor::default();
        unsafe { (self.usb_get_device_descriptor)(self, &mut device_descriptor) }
            .into_with_val(|| device_descriptor)
    }

    pub fn get_config_descriptor(&self) -> Result<UsbConfigDescriptor> {
        let mut config_descriptor = UsbConfigDescriptor::default();
        unsafe { (self.usb_get_config_descriptor)(self, &mut config_descriptor) }
            .into_with_val(|| config_descriptor)
    }

    pub fn get_interface_descriptor(&self) -> Result<UsbInterfaceDescriptor> {
        let mut interface_descriptor = UsbInterfaceDescriptor::default();
        unsafe { (self.usb_get_interface_descriptor)(self, &mut interface_descriptor) }
            .into_with_val(|| interface_descriptor)
    }

    pub fn get_endpoint_descriptor(&self, endpoint_index: u8) -> Result<UsbEndpointDescriptor> {
        let mut endpoint_descriptor = UsbEndpointDescriptor::default();
        unsafe {
            (self.usb_get_endpoint_descriptor)(self, endpoint_index, &mut endpoint_descriptor)
        }
        .into_with_val(|| endpoint_descriptor)
    }

    pub fn get_string_descriptor<'boot>(
        &self,
        boot_services: &'boot BootServices,
        lang_id: u16,
        string_id: u8,
    ) -> Result<PoolString<'boot>> {
        let mut string = ptr::null();
        let status =
            unsafe { (self.usb_get_string_descriptor)(self, lang_id, string_id, &mut string) };
        if status.is_success() {
            let string = PoolString::new(boot_services, string)?;
            status.into_with_val(|| string)
        } else {
            Err(status.into())
        }
    }

    pub fn get_supported_languages(&self) -> Result<&[u16]> {
        // The table is owned by the protocol, so no need to free it. The
        // lifetime of the returned slice is tied to the protocol.
        let mut lang_id_table = ptr::null();

        let mut table_size = 0;
        unsafe {
            (self.usb_get_supported_languages)(self, &mut lang_id_table, &mut table_size)
                .into_with_val(|| slice::from_raw_parts(lang_id_table, usize::from(table_size)))
        }
    }

    pub fn port_reset(&self) -> Result {
        todo!()
    }
}
