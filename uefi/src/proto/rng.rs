//! `Rng` protocol.

pub use uefi_raw::protocol::rng::RngAlgorithmType;

use crate::proto::unsafe_protocol;
use crate::{Result, Status};
use core::{mem, ptr};

/// Rng protocol
#[unsafe_protocol(uefi_raw::protocol::rng::Rng::GUID)]
#[repr(transparent)]
pub struct Rng(uefi_raw::protocol::rng::Rng);

impl Rng {
    /// Returns information about the random number generation implementation.
    pub fn get_info<'buf>(
        &mut self,
        algorithm_list: &'buf mut [RngAlgorithmType],
    ) -> Result<&'buf [RngAlgorithmType], Option<usize>> {
        let mut algorithm_list_size = algorithm_list.len() * mem::size_of::<RngAlgorithmType>();

        unsafe {
            (self.0.get_info)(
                &self.0,
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

        unsafe { (self.0.get_rng)(&self.0, algo, buffer_length, buffer.as_mut_ptr()).into() }
    }
}
