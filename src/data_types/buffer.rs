// TODO
#![allow(missing_docs)]

use crate::{Result, Status};
use core::borrow::BorrowMut;
#[cfg(feature = "exts")]
use {crate::alloc_api::vec::Vec, core::mem::MaybeUninit};

// Use cases we care about:
//
// * Initialized slice: `&mut [T]`
//
// * Uninitialized slice: `&mut [MaybeUninit<T>]`
//
// * Vec: `Vec<T>`. This will internally use uninitialized slices too.
//
// * Limited-size Vec. Something that prevents unlimited growth.

pub trait Buffer<T>: BorrowMut<[T]> {
    fn load<F>(&mut self, mut f: F) -> Result<&mut [T], Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        // TODO: fix for non-u8, add tests for that too
        let mut buf_size = self.borrow().len();

        let status = f(self.borrow_mut().as_mut_ptr(), &mut buf_size);
        status.into_with(
            move || &mut self.borrow_mut()[..buf_size],
            |status| {
                if status == Status::BUFFER_TOO_SMALL {
                    Some(buf_size)
                } else {
                    None
                }
            },
        )
    }
}

impl<T> Buffer<T> for [T] {}

#[cfg(feature = "exts")]
impl<T> Buffer<T> for Vec<T> {
    fn load<F>(&mut self, mut f: F) -> Result<&mut [T], Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        let mut buffer_size = self.len();
        let mut status = f(self.as_mut_ptr(), &mut buffer_size);

        if status == Status::BUFFER_TOO_SMALL {
            // Drop all current elements. This sets the length to zero but
            // does not affect the current allocation.
            self.truncate(0);

            // Reserve the nececessary number of elements. The input length
            // is relative to the vec's `len()`, which we know is zero.
            self.reserve_exact(buffer_size);

            // Get the uninitialized spare capacity (which is the whole
            // capacity in this case).
            let buf: &mut [MaybeUninit<T>] = self.spare_capacity_mut();

            status = f(MaybeUninit::slice_as_mut_ptr(buf), &mut buffer_size);

            if status == Status::SUCCESS {
                // Mark the returned number of elements as initialized.
                unsafe {
                    self.set_len(buffer_size);
                }
            }
        }

        status.into_with(
            move || &mut self.as_mut_slice()[..buffer_size],
            |status| {
                if status == Status::BUFFER_TOO_SMALL {
                    Some(buffer_size)
                } else {
                    None
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_two(buf: *mut u8, buf_size: &mut usize) -> Status {
        unsafe {
            if *buf_size < 2 {
                *buf_size = 2;
                Status::BUFFER_TOO_SMALL
            } else {
                *buf_size = 2;
                buf.write_bytes(0xab, 2);
                Status::SUCCESS
            }
        }
    }

    #[test]
    fn test_vec() {
        // Start out empty, should be resized to fit two.
        let mut buf: Vec<u8> = Vec::new();
        buf.load(load_two).unwrap();
        assert_eq!(buf, [0xab, 0xab]);
    }

    #[test]
    fn test_array() {
        // Too small.
        let mut buf = [0u8; 1];
        let err = buf.load(load_two).unwrap_err();
        assert_eq!(err.status(), Status::BUFFER_TOO_SMALL);

        // Big enough.
        let mut buf = [0u8; 2];
        buf.load(load_two).unwrap();
        assert_eq!(buf, [0xab, 0xab]);
    }

    #[test]
    fn test_slice() {
        // Backing storage for the slice.
        let mut array = [0u8; 2];

        // Too small.
        let buf: &mut [u8] = &mut array[0..1];
        let err = buf.load(load_two).unwrap_err();
        assert_eq!(err.status(), Status::BUFFER_TOO_SMALL);

        // Big enough.
        let buf: &mut [u8] = &mut array[0..2];
        buf.load(load_two).unwrap();
        assert_eq!(buf, [0xab, 0xab]);
    }
}
