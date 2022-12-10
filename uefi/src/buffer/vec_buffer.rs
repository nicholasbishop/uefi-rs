// Everything in this module is gated behind the `alloc` feature.

use super::*;
use crate::ResultExt;
use alloc::vec::Vec;

impl<T> Buffer<T> for Vec<T> {
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnOnce(*mut T, &mut usize) -> Status,
    {
        let mut initialized_len = 0;
        let r = write_impl(f, self.as_mut_ptr(), self.capacity(), &mut initialized_len);
        if r.is_ok() {
            // The existing capacity was big enough.
            self.set_len(initialized_len);
        }
        r
    }

    unsafe fn write_with_auto_resize<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        self.write_with_padded_auto_resize(f, 0)
    }

    unsafe fn write_with_padded_auto_resize<F>(
        &mut self,
        mut f: F,
        extra_len: usize,
    ) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        // Try to write with the current capacity.
        let r = self.write(&mut f);

        // If the write succeeded, or failed with any error except
        // BUFFER_TOO_SMALL, propagate that.
        if r.status() != Status::BUFFER_TOO_SMALL {
            return r;
        }

        // The error data contains the required number of elements.
        let required_len = r.unwrap_err().data().unwrap();

        // Drop all current elements. This sets the length to zero but
        // does not affect the current allocation.
        self.truncate(0);

        // Reserve the nececessary number of elements. The input length
        // is relative to the vec's `len()`, which we know is zero.
        self.reserve_exact(required_len + extra_len);

        // Try writing again with the new capacity.
        self.write(f)
    }
}

// TODO: describe why this exists instead of using `Vec<u64>` or
// similar.
pub struct AlignedByteVec<A: Align> {
    _align: PhantomData<A>,
    data: Vec<u8>,
}

impl<A: Align> AlignedByteVec<A> {
    pub fn new() -> Self {
        Self {
            _align: PhantomData,
            data: Vec::new(),
        }
    }
}

impl<A: Align> Buffer<A> for AlignedByteVec<A> {}
