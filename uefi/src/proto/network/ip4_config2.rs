//! TODO

use core::ptr;
use uefi::{Result, StatusExt};
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::network::ip4_config2::{Ip4Config2DataType, Ip4Config2Protocol};

pub use uefi_raw::protocol::network::ip4_config2::Ip4Config2Policy;

/// TODO
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(Ip4Config2Protocol::GUID)]
pub struct Ip4Config2(Ip4Config2Protocol);

impl Ip4Config2 {
    /// TODO
    pub fn get_policy(&self) -> Result<Ip4Config2Policy> {
        let mut policy = Ip4Config2Policy(-1);
        let mut size = size_of_val(&policy);

        unsafe {
            (self.0.get_data)(
                ptr::from_ref(&self.0).cast_mut(),
                Ip4Config2DataType::POLICY,
                &mut size,
                ptr::from_mut(&mut policy).cast(),
            )
        }
        .to_result_with_val(|| policy)
    }

    /// TODO
    pub fn set_policy(&mut self, policy: Ip4Config2Policy) -> Result {
        unsafe {
            (self.0.set_data)(
                &mut self.0,
                Ip4Config2DataType::POLICY,
                size_of::<Ip4Config2Policy>(),
                ptr::from_ref(&policy).cast(),
            )
        }
        .to_result()
    }
}
