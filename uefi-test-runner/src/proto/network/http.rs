use crate::BootServices;
use log::info;
use uefi::proto::network::http::{Http, HttpConfiguration, HttpServiceBinding, HttpVersion};

// TODO: unwraps
pub fn test(bt: &BootServices) {
    info!("Testing HTTP protocol");
    let handle = bt.get_handle_for_protocol::<HttpServiceBinding>().unwrap();
    let mut http_sb = bt
        .open_protocol_exclusive::<HttpServiceBinding>(handle)
        .unwrap();
    let child_handle = http_sb.create_child().unwrap();
    let mut http = bt.open_protocol_exclusive::<Http>(*child_handle).unwrap();

    let config = HttpConfiguration {
        http_version: HttpVersion::HTTP_VERSION_11,
        time_out_millisec: 1000,
        access_point: Default::default(),
    };
    http.configure(&config).unwrap();

    assert_eq!(config, http.get_configuration().unwrap());
    info!("HTTP configuration: {config:?}");
}
