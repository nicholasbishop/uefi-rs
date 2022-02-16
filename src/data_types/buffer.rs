// TODO
#![allow(missing_docs)]

#[cfg(feature = "exts")]
use crate::alloc_api::vec::Vec;
use crate::{Result, Status};
use core::borrow::BorrowMut;

pub trait Buffer<T>: BorrowMut<[T]> {
    fn is_resizable() -> bool;
    fn resize(&mut self, new_len: usize);

    fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    fn len(&self) -> usize {
        self.borrow().len()
    }

    fn as_slice(&self) -> &[T] {
        self.borrow().as_slice()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        self.borrow_mut()
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.borrow_mut().as_mut_ptr()
    }

    fn load<F>(&mut self, mut f: F) -> Result<&mut [T], Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        let mut buffer_size = self.len();
        let mut status = f(self.as_mut_ptr(), &mut buffer_size);
        if status == Status::BUFFER_TOO_SMALL && Self::is_resizable() {
            self.resize(buffer_size);
            status = f(self.as_mut_ptr(), &mut buffer_size);
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

impl<T> Buffer<T> for [T] {
    fn is_resizable() -> bool {
        false
    }

    fn resize(&mut self, _new_len: usize) {
        panic!("cannot resize array");
    }
}

// TODO: uninitialized buffer?
#[cfg(feature = "exts")]
impl<T: Clone + Default> Buffer<T> for Vec<T> {
    fn is_resizable() -> bool {
        true
    }

    fn resize(&mut self, new_len: usize) {
        self.resize(new_len, Default::default());
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
