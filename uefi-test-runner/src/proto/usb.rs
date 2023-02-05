use log::info;
use uefi::prelude::*;
use uefi::proto::usb::{Class, DescriptorType, UsbIo};
use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    info!("Running UsbIo test");

    let handle = bt
        .get_handle_for_protocol::<UsbIo>()
        .expect("failed to get UsbIo handle");
    let usb_io = bt
        .open_protocol_exclusive::<UsbIo>(handle)
        .expect("failed to open UsbIo protocol");

    let dd = usb_io
        .get_device_descriptor()
        .expect("failed to get device descriptor");
    assert_eq!(dd.descriptor_type, DescriptorType::DEVICE);
    assert_eq!(dd.device_class, Class::USE_INTERFACE_DESCRIPTORS);
    assert_eq!(dd.bcd_usb, 0x0200);

    let cd = usb_io
        .get_config_descriptor()
        .expect("failed to get config descriptor");
    assert_eq!(cd.descriptor_type, DescriptorType::CONFIGURATION);

    let id = usb_io
        .get_interface_descriptor()
        .expect("failed to get interface descriptor");
    assert_eq!(id.descriptor_type, DescriptorType::INTERFACE);

    let ed = usb_io
        .get_endpoint_descriptor(0)
        .expect("failed to get endpoint descriptor");
    assert_eq!(ed.descriptor_type, DescriptorType::ENDPOINT);

    let supported_languages = usb_io
        .get_supported_languages()
        .expect("failed to get supported languages");

    assert_eq!(
        &*usb_io
            .get_string_descriptor(bt, supported_languages[0], dd.str_manufacturer)
            .expect("failed to get manufacturer string"),
        cstr16!("QEMU")
    );
    assert_eq!(
        &*usb_io
            .get_string_descriptor(bt, supported_languages[0], dd.str_product)
            .expect("failed to get product string"),
        cstr16!("QEMU USB Mouse")
    );
    assert_eq!(
        &*usb_io
            .get_string_descriptor(bt, supported_languages[0], dd.str_serial_number)
            .expect("failed to get serial string"),
        cstr16!("89126-0000:00:1d.7-1")
    );
    assert_eq!(
        &*usb_io
            .get_string_descriptor(bt, supported_languages[0], cd.configuration)
            .expect("failed to get configuration string"),
        cstr16!("HID Mouse")
    );

    usb_io.port_reset().expect("failed to reset port");
}
