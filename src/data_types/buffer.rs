// TODO
#![allow(missing_docs)]

#[cfg(feature = "exts")]
use crate::alloc_api::vec::Vec;
use crate::{Result, Status};
use core::borrow::BorrowMut;

/// TODO
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

    #[test]
    // TODO
    fn test_buffer() {
        let mut vbuf: Vec<u8> = Vec::new();
        vbuf.load(|buf, buf_size| unsafe {
            if *buf_size < 2 {
                *buf_size = 2;
                Status::BUFFER_TOO_SMALL
            } else {
                *buf_size = 2;
                buf.write_bytes(0xab, 2);
                Status::SUCCESS
            }
        })
        .unwrap();
        assert_eq!(vbuf, [0xab, 0xab]);
    }
}
