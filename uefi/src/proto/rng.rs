//! `Rng` protocol.

pub use uefi_raw::protocol::rng::RngAlgorithmType;

use crate::proto::unsafe_protocol;
use crate::{Result, Status};
use core::{mem, ptr};

/// Rng protocol
#[repr(C)]
#[unsafe_protocol("3152bca5-eade-433d-862e-c01cdc291f44")]
pub struct Rng {
    // TODO
    imp: uefi_raw::protocol::rng::Rng,
}

impl Rng {
    /// Returns information about the random number generation implementation.
    pub fn get_info<'buf>(
        &mut self,
        algorithm_list: &'buf mut [RngAlgorithmType],
    ) -> Result<&'buf [RngAlgorithmType], Option<usize>> {
        let mut algorithm_list_size = algorithm_list.len() * mem::size_of::<RngAlgorithmType>();

        unsafe {
            (self.imp.get_info)(
                &self.imp,
                &mut algorithm_list_size,
                algorithm_list.as_mut_ptr(),
            )
            .into_with(
                || {
                    let len = algorithm_list_size / mem::size_of::<RngAlgorithmType>();
                    &algorithm_list[..len]
                },
                |status| {
                    if status == Status::BUFFER_TOO_SMALL {
                        Some(algorithm_list_size)
                    } else {
                        None
                    }
                },
            )
        }
    }

    /// Returns the next set of random numbers
    pub fn get_rng(&mut self, algorithm: Option<RngAlgorithmType>, buffer: &mut [u8]) -> Result {
        let buffer_length = buffer.len();

        let algo = match algorithm.as_ref() {
            None => ptr::null(),
            Some(algo) => algo as *const RngAlgorithmType,
        };

        unsafe { (self.imp.get_rng)(&self.imp, algo, buffer_length, buffer.as_mut_ptr()).into() }
    }
}
