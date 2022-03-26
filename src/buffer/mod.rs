// TODO
#![allow(missing_docs)]

// Note on `Debug` impls for the structs in this module: these all just
// print the contained elements without the name of the struct,
// e.g. `[a, b, c]`. That matches the behavior of `Vec`.

use crate::{Error, Result, Status};
use core::fmt::{self, Debug, Formatter};
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};

/// Generic buffer interface for use with UEFI functions that output a
/// slice of data. This can be implemented on a dynamic heap-allocated
/// type like a [`Vec`], or on a type that combines TODO.
///
/// Implementers of the trait must provide both a slice to write into
/// and store the length of the data. [`Vec`] directly implements this
/// trait since it knows its own length. The trait cannot be directly
/// implemented on `&mut [T]`
pub trait Buffer<T>: Deref<Target = [T]> + DerefMut {
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status;
}

fn write_impl<T, F>(
    f: &mut F,
    data: *mut T,
    max_len: usize,
    real_len: &mut usize,
) -> Result<(), Option<usize>>
where
    F: FnMut(*mut T, &mut usize) -> Status,
{
    // Get the maximum output size in bytes.
    let mut size_in_bytes = max_len * mem::size_of::<T>();

    // Try to write out the data. `size_in_bytes` will be updated to the
    // actual size of the data, regardless of whether the buffer is big
    // enough. The function returns `BUFFER_TOO_SMALL` if the buffer
    // isn't big enough.
    let status = f(data, &mut size_in_bytes);

    // Convert the `size_in_bytes` to the number of `T`.
    debug_assert_eq!(size_in_bytes % mem::size_of::<T>(), 0);
    let size_in_elements = size_in_bytes / mem::size_of::<T>();

    // If successful, update the buffer's length.
    if status.is_success() {
        *real_len = size_in_elements;
    }

    match status {
        Status::SUCCESS => Ok(()),
        Status::BUFFER_TOO_SMALL => Err(Error::new(status, Some(size_in_elements))),
        _ => Err(Error::new(status, None)),
    }
}

pub type EmptyBuffer<T> = ArrayBuffer<T, 0>;

pub struct SliceBuffer<'a, T> {
    slice: &'a mut [T],
    len: usize,
}

impl<'a, T> Buffer<T> for SliceBuffer<'a, T> {
    unsafe fn write<F>(&mut self, mut f: F) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        write_impl(
            &mut f,
            self.slice.as_mut_ptr(),
            self.slice.len(),
            &mut self.len,
        )
    }
}

impl<'a, T: Debug> Debug for SliceBuffer<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, T> Deref for SliceBuffer<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.slice[..self.len]
    }
}

impl<'a, T> DerefMut for SliceBuffer<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.slice[..self.len]
    }
}

impl<'a, T> SliceBuffer<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        Self { slice, len: 0 }
    }
}

pub struct ArrayBuffer<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> Buffer<T> for ArrayBuffer<T, N> {
    unsafe fn write<F>(&mut self, mut f: F) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        write_impl(
            &mut f,
            MaybeUninit::slice_as_mut_ptr(self.array.as_mut_slice()),
            self.array.len(),
            &mut self.len,
        )
    }
}

impl<T: Debug, const N: usize> Debug for ArrayBuffer<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, const N: usize> ArrayBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            // Safety: as described in the `MaybeUninit` docs, it is
            // safe to use `assume_init` here because the `MaybeUninit`
            // elements in the array do not require initialization.
            array: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
}

impl<T, const N: usize> Deref for ArrayBuffer<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        let slice: &[MaybeUninit<T>] = &self.array[..self.len];
        // Safety: `slice` contains only the initialized part of the array.
        unsafe { MaybeUninit::slice_assume_init_ref(slice) }
    }
}

impl<T, const N: usize> DerefMut for ArrayBuffer<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        let slice: &mut [MaybeUninit<T>] = &mut self.array[..self.len];
        // Safety: `slice` contains only the initialized part of the array.
        unsafe { MaybeUninit::slice_assume_init_mut(slice) }
    }
}

#[cfg(feature = "exts")]
mod vec_buffer;

#[cfg(feature = "exts")]
pub use vec_buffer::VecWithMaxLen;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc_api::format;

    #[cfg(feature = "exts")]
    use alloc_api::vec::Vec;

    fn write_n<B: Buffer<u32>>(n: usize, buf: &mut B) -> Result<(), Option<usize>> {
        unsafe {
            buf.write(|data, size_in_bytes| {
                let required_size_in_bytes = n * mem::size_of::<u32>();
                if *size_in_bytes < required_size_in_bytes {
                    *size_in_bytes = required_size_in_bytes;
                    return Status::BUFFER_TOO_SMALL;
                }

                for i in 0..n {
                    let elem = data.add(i);
                    elem.write(i as u32);
                }

                *size_in_bytes = required_size_in_bytes;
                Status::SUCCESS
            })
        }
    }

    struct CappedAt3(bool);

    fn check_buffer<B: Buffer<u32> + Debug>(buf: &mut B, capped_at_3: CappedAt3) {
        // If the function passed to `write` returns any error other
        // than `BUFFER_TOO_SMALL`, that error should be propagated.
        unsafe {
            assert_eq!(
                buf.write(|_, _| Status::UNSUPPORTED).unwrap_err().status(),
                Status::UNSUPPORTED
            );
            assert!(buf.is_empty());
        }

        // Write out two elements and verify the result. Even if the
        // buffer is fixed-size with more than two elements, only a
        // two-element slice will be returned.
        write_n(2, buf).unwrap();
        assert_eq!(**buf, [0, 1]);

        // Write out three elements and verify the result.
        write_n(3, buf).unwrap();
        assert_eq!(**buf, [0, 1, 2]);

        // Check the `Debug` formatting.
        assert_eq!(format!("{:?}", buf), "[0, 1, 2]");

        // Try writing out four elements. If the buffer can't hold that
        // many, expect a `BUFFER_TOO_SMALL` error.
        let r = write_n(4, buf);
        if capped_at_3.0 {
            assert_eq!(r.err().unwrap().status(), Status::BUFFER_TOO_SMALL);
        } else {
            r.unwrap();
            assert_eq!(**buf, [0, 1, 2, 3]);
        }
    }

    #[test]
    fn test_slice_buffer() {
        let slice = &mut [0, 0, 0];
        let mut buf = SliceBuffer::new(slice);
        check_buffer(&mut buf, CappedAt3(true));
    }

    #[test]
    fn test_array_buffer() {
        let mut buf = ArrayBuffer::<u32, 3>::new();
        check_buffer(&mut buf, CappedAt3(true));
    }

    #[cfg(feature = "exts")]
    #[test]
    fn test_vec_buffer() {
        let mut buf = Vec::new();
        check_buffer(&mut buf, CappedAt3(false));
    }

    #[cfg(feature = "exts")]
    #[test]
    fn test_vec_with_max_len_buffer() {
        let mut buf = VecWithMaxLen::new(3);
        check_buffer(&mut buf, CappedAt3(true));
    }
}
