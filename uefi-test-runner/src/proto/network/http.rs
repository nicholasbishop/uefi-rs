use core::time::Duration;
use log::info;
use uefi::proto::network::http::{
    Http, HttpConfiguration, HttpHeader, HttpMethod, HttpRequest, HttpServiceBinding, HttpVersion,
};
use uefi::proto::network::ip4_config2::{Ip4Config2, Ip4Config2Policy};
use uefi::{boot, cstr16, cstr8};

// TODO: unwraps
pub fn test() {
    info!("Testing HTTP protocol");

    // TODO
    let handle = boot::get_handle_for_protocol::<Ip4Config2>().unwrap();
    let mut ipconfig = boot::open_protocol_exclusive::<Ip4Config2>(handle).unwrap();
    ipconfig.set_policy(Ip4Config2Policy::DHCP).unwrap();
    info!("Ip4Config2 policy: {:?}", ipconfig.get_policy().unwrap());

    boot::stall(4_000_000);

    let handle = boot::get_handle_for_protocol::<HttpServiceBinding>().unwrap();
    let mut http_sb = boot::open_protocol_exclusive::<HttpServiceBinding>(handle).unwrap();
    let child_handle = http_sb.create_child().unwrap();
    let mut http = boot::open_protocol_exclusive::<Http>(*child_handle).unwrap();

    let config = HttpConfiguration {
        http_version: HttpVersion::HTTP_VERSION_11,
        // TODO
        timeout: Duration::from_secs(10),
        access_point: Default::default(),
    };
    http.configure(&config).unwrap();

    assert_eq!(config, http.get_configuration().unwrap());
    info!("HTTP configuration: {config:?}");

    http.send_request_sync(HttpRequest {
        method: HttpMethod::GET,
        url: cstr16!("http://example.com"),
        headers: &[HttpHeader::new(
            cstr8!("Host"),
            cstr8!("http://example.com"),
        )],
        body: &[],
    })
    .unwrap();
}
