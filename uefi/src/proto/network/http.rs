// TODO
#![allow(missing_docs)]

use crate::data_types::Ipv4Address;
use crate::{Handle, Result, StatusExt};
use core::ops::Deref;
use core::ptr;
use log::error;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::driver::ServiceBindingProtocol;
use uefi_raw::protocol::network::http::{HttpConfigData, HttpProtocol};

pub use uefi_raw::protocol::network::http::{HttpV4AccessPoint, HttpV6AccessPoint, HttpVersion};

#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(HttpProtocol::GUID)]
pub struct Http(HttpProtocol);

impl Http {
    // TODO: pretty sure we need to think about pinning here
    pub fn get_configuration(&self) -> Result<HttpConfiguration> {
        let mut config = HttpConfigData::default();
        let mut access_point = HttpV6AccessPoint::default();
        // TODO: deal with dealloc by making a better config type
        config.access_point.ipv6_node = &mut access_point;

        unsafe { (self.0.get_mode_data)(&self.0, &mut config) }.to_result_with_val(|| {
            HttpConfiguration {
                http_version: config.http_version,
                time_out_millisec: config.time_out_millisec,
                access_point: if config.local_addr_is_ipv6 {
                    let node = unsafe { &*config.access_point.ipv6_node };
                    HttpAccessPoint::IpV6(HttpV6AccessPoint {
                        local_address: node.local_address,
                        local_port: node.local_port,
                    })
                } else {
                    let node = unsafe { &*config.access_point.ipv4_node };
                    HttpAccessPoint::IpV4(HttpV4AccessPoint {
                        use_default_addr: node.use_default_addr,
                        local_address: node.local_address,
                        local_subnet: node.local_subnet,
                        local_port: node.local_port,
                    })
                },
            }
        })
    }

    pub fn configure(&mut self, config: &HttpConfiguration) -> Result {
        let mut raw_config = HttpConfigData {
            http_version: config.http_version,
            time_out_millisec: config.time_out_millisec,
            ..Default::default()
        };

        match config.access_point {
            HttpAccessPoint::IpV4(ap) => {
                raw_config.local_addr_is_ipv6 = false;
                raw_config.access_point.ipv4_node = &ap;
            }
            HttpAccessPoint::IpV6(ap) => {
                raw_config.local_addr_is_ipv6 = true;
                raw_config.access_point.ipv6_node = &ap;
            }
        }

        unsafe { (self.0.configure)(&mut self.0, &raw_config) }.to_result()
    }

    pub fn request(&mut self, request: HttpRequest) -> Result<HttpToken> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HttpConfiguration {
    pub http_version: HttpVersion,
    // TODO: field naming
    pub time_out_millisec: u32,
    pub access_point: HttpAccessPoint,
}

// TODO
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum HttpAccessPoint {
    IpV4(HttpV4AccessPoint),
    IpV6(HttpV6AccessPoint),
}

impl Default for HttpAccessPoint {
    fn default() -> Self {
        // TODO
        Self::IpV4(HttpV4AccessPoint {
            use_default_addr: true,
            local_address: Ipv4Address::default(),
            local_subnet: Ipv4Address::default(),
            local_port: 80,
        })
    }
}

pub struct HttpRequest {

    // TODO
}

pub struct HttpToken {
    // TODO
}

#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(HttpProtocol::SERVICE_BINDING_GUID)]
pub struct HttpServiceBinding(ServiceBindingProtocol);

impl HttpServiceBinding {
    pub fn create_child(&mut self) -> Result<ServiceBindingHandle> {
        let mut child_handle = ptr::null_mut();

        unsafe {
            (self.0.create_child)(&mut self.0, &mut child_handle).to_result_with_val(|| {
                // OK to unwrap: `create_child` returned SUCCESS, so the handle
                // is valid.
                let child_handle = Handle::from_ptr(child_handle).expect("invalid child handle");
                ServiceBindingHandle {
                    service_binding: &self.0,
                    child_handle,
                }
            })
        }
    }
}

#[derive(Debug)]
pub struct ServiceBindingHandle<'a> {
    service_binding: &'a ServiceBindingProtocol,
    child_handle: Handle,
}

impl<'a> Deref for ServiceBindingHandle<'a> {
    type Target = Handle;

    fn deref(&self) -> &Handle {
        &self.child_handle
    }
}

impl<'a> Drop for ServiceBindingHandle<'a> {
    fn drop(&mut self) {
        let sb_ptr: *const _ = self.service_binding;
        let status = unsafe {
            (self.service_binding.destroy_child)(sb_ptr.cast_mut(), self.child_handle.as_ptr())
        };
        if !status.is_success() {
            // Log the error, but otherwise ignore it since we can't propagate
            // an error from drop.
            error!("failed to destroy service binding child: {status}");
        }
    }
}
