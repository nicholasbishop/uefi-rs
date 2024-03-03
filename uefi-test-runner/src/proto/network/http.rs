use crate::{cstr16, BootServices};
use core::time::Duration;
use log::info;
use uefi::proto::network::http::{
    Http, HttpConfiguration, HttpMethod, HttpRequest, HttpServiceBinding, HttpVersion,
};

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
        timeout: Duration::from_secs(1),
        access_point: Default::default(),
    };
    http.configure(&config).unwrap();

    assert_eq!(config, http.get_configuration().unwrap());
    info!("HTTP configuration: {config:?}");

    http.send_request_sync(
        bt,
        HttpRequest {
            method: HttpMethod::GET,
            url: cstr16!("http://example.com"),
            headers: &[],
            body: &[],
        },
    )
    .unwrap();
}
