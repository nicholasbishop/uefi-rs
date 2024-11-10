use core::time::Duration;
use log::info;
use uefi::proto::network::http::{
    Http, HttpConfiguration, HttpMethod, HttpRequest, HttpServiceBinding, HttpVersion,
};
use uefi::{boot, cstr16};

// TODO: uefi wrappers
use uefi_raw::protocol::network::ip4_config2::Ip4Config2Protocol;

// TODO: unwraps
pub fn test() {
    info!("Testing HTTP protocol");

    let handle = boot::get_handle_for_protocol::<HttpServiceBinding>().unwrap();

    Ip4Config2Protocol::pub get_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        data_type: Ip4Config2DataType,
        data_size: *mut usize,
        data: *mut c_void,
    ) -> Status,

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
