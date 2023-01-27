use log::info;
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

    let cd = usb_io
        .get_config_descriptor()
        .expect("failed to get config descriptor");
    info!("{:?}", cd);

    let id = usb_io
        .get_interface_descriptor()
        .expect("failed to get interface descriptor");
    info!("{:?}", id);

    let ed = usb_io
        .get_endpoint_descriptor(0)
        .expect("failed to get endpoint descriptor");
    info!("{:?}", ed);

    let sl = usb_io
        .get_supported_languages()
        .expect("failed to get supported languages");
    info!("{:?}", sl);

    info!(
        "{}",
        &*usb_io
            .get_string_descriptor(bt, sl[0], dd.str_manufacturer)
            .unwrap()
    );
    info!(
        "{}",
        &*usb_io
            .get_string_descriptor(bt, sl[0], dd.str_product)
            .unwrap()
    );
    info!(
        "{}",
        &*usb_io
            .get_string_descriptor(bt, sl[0], dd.str_serial_number)
            .unwrap()
    );
}
