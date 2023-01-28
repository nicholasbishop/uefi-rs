use log::info;
use uefi::prelude::*;
use uefi::proto::usb::UsbIo;
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
    info!("{:?}", dd);

    usb_io
        .get_config_descriptor()
        .expect("failed to get config descriptor");

    // TODO: check some values on this one?
    let id = usb_io
        .get_interface_descriptor()
        .expect("failed to get interface descriptor");
    info!("{:?}", id);

    usb_io
        .get_endpoint_descriptor(0)
        .expect("failed to get endpoint descriptor");

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

    usb_io.port_reset().expect("failed to reset port");
}
