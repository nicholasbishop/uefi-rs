// TODO
#![allow(missing_docs)]

use crate::boot::{self, EventType, MemoryType, Tpl};
use crate::data_types::Ipv4Address;
use crate::{CStr16, CStr8, Error, Event, Handle, Result, Status, StatusExt};
use core::ffi::c_void;
use core::mem;
use core::ops::Deref;
use core::ptr::{self, NonNull};
use core::time::Duration;
use log::{error, info};
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::driver::ServiceBindingProtocol;
use uefi_raw::protocol::network::http as http_raw;
use uefi_raw::protocol::network::http::{HttpResponseData, HttpStatusCode};

// TODO, not pub for most of these
pub use uefi_raw::protocol::network::http::{
    HttpMessage, HttpMethod, HttpRequestData, HttpRequestOrResponse, HttpToken, HttpV4AccessPoint,
    HttpV6AccessPoint, HttpVersion,
};

#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(http_raw::HttpProtocol::GUID)]
pub struct Http(http_raw::HttpProtocol);

impl Http {
    pub fn get_configuration(&self) -> Result<HttpConfiguration> {
        // Allocate memory to pass into `get_mode_data`.
        let mut config = http_raw::HttpConfigData::default();
        let mut access_point = HttpV6AccessPoint::default();
        config.access_point.ipv6_node = &mut access_point;

        unsafe { (self.0.get_mode_data)(&self.0, &mut config) }.to_result_with_val(|| {
            // Convert from the raw `HttpConfigData` type to
            // `HttpConfiguration`. The latter uses a Rust enum for the access
            // point, so it does not require additional allocations or `unsafe`
            // to access.
            HttpConfiguration {
                http_version: config.http_version,
                timeout: Duration::from_millis(config.time_out_millisec.into()),
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
        let mut raw_config = http_raw::HttpConfigData {
            http_version: config.http_version,
            time_out_millisec: config
                .timeout
                .as_millis()
                .try_into()
                .map_err(|_| Status::INVALID_PARAMETER)?,
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

        // SAFETY: the data in `raw_config` is copied internally by the driver,
        // so it's OK to pass in pointers to short-lived data.
        unsafe { (self.0.configure)(&mut self.0, &raw_config) }.to_result()
    }

    // TODO, not sure about API yet.
    // TODO: add an asyn request or something less safe?
    pub fn send_request_sync(&mut self, request: HttpRequest) -> Result<()> {
        let request_data = HttpRequestData {
            method: request.method,
            url: request.url.as_ptr().cast(),
        };

        let headers: *mut http_raw::HttpHeader = boot::allocate_pool(
            // TODO: maybe have a global somewhere for memtype?
            MemoryType::LOADER_DATA,
            mem::size_of::<http_raw::HttpHeader>() * request.headers.len(),
        )?
        .cast()
        .as_ptr();
        for i in 0..request.headers.len() {
            let dst = unsafe { &mut *headers.add(i) };
            dst.field_name = request.headers[i].name.as_ptr().cast();
            dst.field_value = request.headers[i].value.as_ptr().cast();
        }

        let mut message = HttpMessage {
            data: HttpRequestOrResponse {
                request: &request_data,
            },
            header_count: request.headers.len(),
            headers,
            body_length: request.body.len(),
            body: request.body.as_ptr().cast(),
        };

        let mut is_done = false;
        let is_done_ptr = ptr::from_mut(&mut is_done);

        let event = unsafe {
            boot::create_event(
                EventType::NOTIFY_SIGNAL,
                Tpl::NOTIFY,
                Some(done_callback),
                NonNull::new(is_done_ptr.cast()),
            )?
        };

        let mut token = HttpToken {
            event: event.as_ptr(),
            status: Status::NOT_READY,
            message: &mut message,
        };
        let token_ptr = ptr::from_mut(&mut token);
        // TODO
        info!("token: {token:?}");
        info!("message: {:?}", unsafe { &*token.message });

        unsafe { (self.0.request)(&mut self.0, &mut token) }.to_result()?;

        // Wait for the request to finish.
        while unsafe { !is_done_ptr.read_volatile() } {
            info!("not yet"); // TODO
            self.poll()?;
        }

        // TODO
        info!("request done");

        // Check token status.
        let status = unsafe { token_ptr.read_volatile() }.status;
        if status != Status::SUCCESS {
            return Err(Error::from(status));
        }

        // TODO: clean up the event.

        self.read_response()
    }

    // TODO: api
    fn read_response(&mut self) -> Result<()> {
        // TODO: dedup with request
        let mut is_done = false;
        let is_done_ptr = ptr::from_mut(&mut is_done);

        let event = unsafe {
            boot::create_event(
                EventType::NOTIFY_SIGNAL,
                Tpl::NOTIFY,
                Some(done_callback),
                NonNull::new(is_done_ptr.cast()),
            )?
        };

        let mut response = HttpResponseData {
            status_code: HttpStatusCode::STATUS_UNSUPPORTED,
        };

        // TODO: make sure all allocations are freed.
        // TODO: make alloc required?
        let body_len = 4096;
        let body = boot::allocate_pool(
            // TODO: maybe have a global somewhere for memtype?
            MemoryType::LOADER_DATA,
            // TODO: use page alloc instead?
            body_len,
        )?;

        let mut message = HttpMessage {
            data: HttpRequestOrResponse {
                response: &mut response,
            },
            // TODO
            header_count: 0,
            headers: ptr::null_mut(),
            body_length: body_len,
            body: body.as_ptr().cast(),
        };

        let mut token = HttpToken {
            event: event.as_ptr(),
            status: Status::NOT_READY,
            message: &mut message,
        };
        let token_ptr = ptr::from_mut(&mut token);

        info!("awaiting response");
        unsafe { (self.0.response)(&mut self.0, &mut token) }.to_result()?;
        info!("response call done");

        // Wait for the response to finish.
        while unsafe { !is_done_ptr.read_volatile() } {
            info!("not yet"); // TODO
            self.poll()?;
        }

        // TODO: handle body buf too small.

        // TODO
        info!("response done");

        info!("http code: {:?}", unsafe {
            ptr::from_mut(&mut response).read_volatile()
        });
        info!("num headers: {}", message.header_count);
        for i in 0..message.header_count {
            let header = unsafe { message.headers.add(i) };
            let header = unsafe { &*header };
            let name = unsafe { CStr8::from_ptr(header.field_name.cast()) };
            let val = unsafe { CStr8::from_ptr(header.field_value.cast()) };
            info!("header {i}: {name}: {val}");
        }

        // Check token status.
        let token = unsafe { token_ptr.read_volatile() };
        let status = token.status;
        if status != Status::SUCCESS {
            return Err(Error::from(status));
        }

        todo!()
    }

    // TODO: pub?
    fn poll(&mut self) -> Result<()> {
        unsafe { (self.0.poll)(&mut self.0) }.to_result()
    }
}

unsafe extern "efiapi" fn done_callback(_event: Event, context: Option<NonNull<c_void>>) {
    if let Some(context) = context {
        let is_done: *mut bool = context.as_ptr().cast();
        *is_done = true;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HttpConfiguration {
    pub http_version: HttpVersion,
    pub timeout: Duration,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HttpHeader<'a> {
    pub name: &'a CStr8,
    pub value: &'a CStr8,
}

impl<'a> HttpHeader<'a> {
    #[must_use]
    pub const fn new(name: &'a CStr8, value: &'a CStr8) -> Self {
        Self { name, value }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HttpRequest<'a> {
    pub method: HttpMethod,
    pub url: &'a CStr16,
    pub headers: &'a [HttpHeader<'a>],
    pub body: &'a [u8],
}

#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(http_raw::HttpProtocol::SERVICE_BINDING_GUID)]
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
