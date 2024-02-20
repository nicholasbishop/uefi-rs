use log::info;
use uefi::boot;
use uefi::proto::network::http::{Http, HttpConfiguration, HttpServiceBinding, HttpVersion};

// TODO: unwraps
pub fn test() {
    info!("Testing HTTP protocol");
    let handle = boot::get_handle_for_protocol::<HttpServiceBinding>().unwrap();
    let mut http_sb = boot::open_protocol_exclusive::<HttpServiceBinding>(handle).unwrap();
    let child_handle = http_sb.create_child().unwrap();
    let mut http = boot::open_protocol_exclusive::<Http>(*child_handle).unwrap();

    let config = HttpConfiguration {
        http_version: HttpVersion::HTTP_VERSION_11,
        time_out_millisec: 1000,
        access_point: Default::default(),
    };
    http.configure(&config).unwrap();

    assert_eq!(config, http.get_configuration().unwrap());
    info!("HTTP configuration: {config:?}");
}
