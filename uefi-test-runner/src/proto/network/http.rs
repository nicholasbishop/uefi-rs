use core::time::Duration;
use log::info;
use uefi::proto::network::http::{
    Http, HttpConfiguration, HttpMethod, HttpRequest, HttpServiceBinding, HttpVersion,
};
use uefi::{boot, cstr16};

// TODO: unwraps
pub fn test() {
    info!("Testing HTTP protocol");
    let handle = boot::get_handle_for_protocol::<HttpServiceBinding>().unwrap();
    let mut http_sb = boot::open_protocol_exclusive::<HttpServiceBinding>(handle).unwrap();
    let child_handle = http_sb.create_child().unwrap();
    let mut http = boot::open_protocol_exclusive::<Http>(*child_handle).unwrap();

    let config = HttpConfiguration {
        http_version: HttpVersion::HTTP_VERSION_11,
        timeout: Duration::from_secs(1),
        access_point: Default::default(),
    };
    http.configure(&config).unwrap();

    assert_eq!(config, http.get_configuration().unwrap());
    info!("HTTP configuration: {config:?}");

    http.send_request_sync(HttpRequest {
        method: HttpMethod::GET,
        url: cstr16!("http://example.com"),
        headers: &[],
        body: &[],
    })
    .unwrap();
}
