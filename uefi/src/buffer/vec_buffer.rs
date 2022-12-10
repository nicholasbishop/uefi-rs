// Everything in this module is gated behind the `alloc` feature.

use super::*;
use crate::ResultExt;
use alloc::{vec, vec::Vec};

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

// TODO: dedup with public Align trait
/// Calculate the offset from `val` necessary to make it aligned,
/// rounding up. For example, if `val` is 1 and the alignment is 8,
/// this will return 7. Returns 0 if `val == 0`.
#[must_use]
fn offset_up_to_alignment(val: usize, alignment: usize) -> usize {
    assert!(alignment != 0);
    let r = val % alignment;
    if r == 0 {
        0
    } else {
        alignment - r
    }
}

/// TODO
// TODO: describe why this exists instead of using `Vec<u64>` or
// similar.
pub struct AlignedByteVec<A: Align> {
    _align: PhantomData<A>,
    data: Vec<u8>,
}

impl<A: Align> AlignedByteVec<A> {
    /// TODO
    pub fn new() -> Self {
        Self {
            _align: PhantomData,
            data: vec![0; mem::align_of::<A>()],
        }
    }

    fn alignment_offset(&self) -> usize {
        let ptr = self.data.as_ptr();
        offset_up_to_alignment(ptr as usize, mem::align_of::<A>())
    }
}

impl<A: Align> Deref for AlignedByteVec<A> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.data[self.alignment_offset()..]
    }
}

impl<A: Align> DerefMut for AlignedByteVec<A> {
    fn deref_mut(&mut self) -> &mut [u8] {
        let alignment_offset = self.alignment_offset();
        &mut self.data[alignment_offset..]
    }
}

impl<A: Align> Buffer<u8, A> for AlignedByteVec<A> {
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnOnce(*mut u8, &mut usize) -> Status,
    {
        let mut initialized_len = 0;
        let ptr = self.data.as_mut_ptr();
        let capacity = self.data.capacity();
        let offset = offset_up_to_alignment(ptr as usize, mem::align_of::<A>());
        assert!(capacity >= offset);
        let r = write_impl(f, ptr.add(offset), capacity - offset, &mut initialized_len);
        if r.is_ok() {
            // The existing capacity was big enough.
            self.data.set_len(initialized_len + offset);
        }
        r
    }
}
