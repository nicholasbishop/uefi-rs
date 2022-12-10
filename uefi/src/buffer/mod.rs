use crate::{Error, Result, Status};
use core::fmt::{self, Debug, Formatter};
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::slice;

/// Generic buffer interface for use with UEFI functions that output a
/// data into a buffer. Its primary purpose is to allow UEFI functions
/// to be wrapped in a single Rust method that can work with either a
/// statically-sized buffer or a dynamic-allocated [`Vec`]. It also
/// helps ensure that callers of such methods don't accidentally read
/// past the end of the initialized data.
///
/// Implementers of the trait must provide a slice to write into and
/// also store the length of the initialized data. [`Vec`] directly
/// implements this trait since it knows its own length. The trait
/// cannot be directly implemented on `&mut [T]`, so a [`SliceBuffer`]
/// wrapper type is provided.
///
/// TODO: examples of SliceBuffer, ArrayBuffer, and Vec.
///
/// [`Vec`]: https://doc.rust-lang.org/alloc/vec/struct.Vec.html
pub trait Buffer<T>: Deref<Target = [T]> + DerefMut {
    /// Attempt to write data into the buffer. Data is always written
    /// starting at the beginning of the buffer, overwriting any
    /// existing data.
    ///
    /// # `f` argument
    ///
    /// The `f` function argument is what actually generates the data,
    /// usually a closure that ends up calling a UEFI function. The
    /// function signature looks like this:
    ///
    /// ```
    /// # use uefi::Status;
    /// # type T = ();
    /// fn f(buf: *mut T, size_in_bytes: &mut usize) -> Status {
    ///   // ...
    /// # todo!()
    /// }
    /// ```
    ///
    /// ## Arguments of `f`
    ///
    /// * `buf`: a pointer to the beginning of the output buffer. If not
    ///   null, the pointer must be a valid location to write to (a
    ///   valid and aligned allocation), but does not need to contain
    ///   initialized data (it is guaranteed that the data is only
    ///   written to, never read from). It is also allowed for `buf` to
    ///   be null to just get the size.
    /// * `size_in_bytes`: an in+out parameter for the buffer size.
    ///   * Input: contains the size in bytes of the output buffer (or 0
    ///     if `buf` is null).
    ///   * Output: if `f` returns [`Status::SUCCESS`] then
    ///     `size_in_bytes` will contain the number of bytes written. If
    ///     `f` returns [`Status::BUFFER_TOO_SMALL`] then
    ///     `size_in_bytes` will contain the total size in bytes of the
    ///     data that would have been written if the buffer were large
    ///     enough. For any other return value of `f`, the value of
    ///     `size_in_bytes` is unspecified.
    ///
    /// ## Return value of `f`
    ///
    /// The return value of `f` must be [`Status::SUCCESS`] if the data
    /// was successfully written. In this case the output buffer will
    /// contain `size_in_bytes` bytes of initialized data, and `write`
    /// will return `Ok(())`. If the size of the output buffer was not
    /// big enough, `f` must return [`Status::BUFFER_TOO_SMALL`], and
    /// `write` will return `Err(Some(size_in_bytes))`. For any other
    /// return value from `f`, `write` will return `Err(None)`.
    ///
    /// # Calling `f`
    ///
    /// Implementors may call `f` multiple times. This is useful when
    /// allocation is supported, to grow the underlying buffer if it is
    /// not yet big enough. If `f` is called multiple times, it is not
    /// guaranteed that the required size of the data will be the same
    /// every time. In other words, implementors cannot assume that
    /// calling `f` to get the size, then allocating space for that
    /// number of bytes, and calling `f` again, will necessarily
    /// succeed. See [`BootServices::memory_map`] for an example of
    /// this.
    ///
    /// [`BootServices::memory_map`]: crate::table::boot::BootServices::memory_map
    ///
    /// # Safety
    ///
    /// Callers must ensure that the function passed in meets all of the
    /// requirements specified above.
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnOnce(*mut T, &mut usize) -> Status;

    /// TODO
    ///
    /// # Safety
    ///
    /// TODO
    unsafe fn write_with_auto_resize<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        self.write_with_padded_auto_resize(f, 0)
    }

    /// TODO
    ///
    /// # Safety
    ///
    /// TODO
    unsafe fn write_with_padded_auto_resize<F>(
        &mut self,
        f: F,
        _extra_len: usize,
    ) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        self.write(f)
    }
}

fn write_impl<T, F>(
    f: F,
    data: *mut T,
    max_len: usize,
    initialized_len: &mut usize,
) -> Result<(), Option<usize>>
where
    F: FnOnce(*mut T, &mut usize) -> Status,
{
    // Get the maximum output size in bytes.
    let mut size_in_bytes = max_len * mem::size_of::<T>();

    // Try to write out the data. `size_in_bytes` will be updated to the
    // actual size of the data if `f` returns either `SUCCESS` or
    // `BUFFER_TOO_SMALL`.
    let status = f(data, &mut size_in_bytes);

    // Convert the `size_in_bytes` to the number of `T`.
    assert_eq!(size_in_bytes % mem::size_of::<T>(), 0);
    let size_in_elements = size_in_bytes / mem::size_of::<T>();

    // If successful, update the buffer's length.
    if status.is_success() {
        *initialized_len = size_in_elements;
    }

    match status {
        Status::SUCCESS => Ok(()),
        Status::BUFFER_TOO_SMALL => Err(Error::new(status, Some(size_in_elements))),
        _ => Err(Error::new(status, None)),
    }
}

/// Wrapper that implements [`Buffer`] for a mutable slice.
pub struct SliceBuffer<'a, T: 'a> {
    // Internally use a pointer rather than `&mut [T]` to support using
    // potentially-uninitialized data. We could use `&mut
    // [MaybeUninit<T>]`, but it's not considered sound to cast to that
    // from `&mut [T]`.
    data: *mut T,
    initialized_len: usize,
    max_len: usize,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> Buffer<T> for SliceBuffer<'a, T> {
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnOnce(*mut T, &mut usize) -> Status,
    {
        write_impl(f, self.data, self.max_len, &mut self.initialized_len)
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
        // Safety: the data up to `initialized_len` is guaranteed to be
        // valid.
        unsafe { slice::from_raw_parts(self.data, self.initialized_len) }
    }
}

impl<'a, T> DerefMut for SliceBuffer<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        // Safety: the data up to `initialized_len` is guaranteed to be
        // valid.
        unsafe { slice::from_raw_parts_mut(self.data, self.initialized_len) }
    }
}

impl<'a, T> SliceBuffer<'a, T> {
    /// Create a new `SliceBuffer` from a slice.
    pub fn new(slice: &'a mut [T]) -> Self {
        Self {
            data: slice.as_mut_ptr(),
            max_len: slice.len(),
            initialized_len: 0,
            phantom: PhantomData,
        }
    }

    /// Create a new `SliceBuffer` from a [`MaybeUninit`] slice.
    pub fn from_maybe_uninit(slice: &'a mut [MaybeUninit<T>]) -> Self {
        Self {
            data: slice.as_mut_ptr().cast(),
            max_len: slice.len(),
            initialized_len: 0,
            phantom: PhantomData,
        }
    }

    /// Create a new `SliceBuffer` from a raw pointer and length. The
    /// `len` is the number of elements of `T` in `data` (not the number
    /// of bytes).
    ///
    /// # Safety
    ///
    /// The pointer must be a valid location to write to (a valid and
    /// aligned allocation), but does not need to contain initialized
    /// data.
    pub unsafe fn from_ptr(data: *mut T, len: usize) -> Self {
        Self {
            data,
            max_len: len,
            initialized_len: 0,
            phantom: PhantomData,
        }
    }
}

/// Wrapper that implements [`Buffer`] for a mutable array.
pub struct ArrayBuffer<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    initialized_len: usize,
}

impl<T, const N: usize> Buffer<T> for ArrayBuffer<T, N> {
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnOnce(*mut T, &mut usize) -> Status,
    {
        write_impl(
            f,
            MaybeUninit::slice_as_mut_ptr(self.array.as_mut_slice()),
            self.array.len(),
            &mut self.initialized_len,
        )
    }
}

impl<T: Debug, const N: usize> Debug for ArrayBuffer<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, const N: usize> ArrayBuffer<T, N> {
    /// Create a new `ArrayBuffer`.
    pub const fn new() -> Self {
        Self {
            // Safety: as described in the `MaybeUninit` docs, it is
            // safe to use `assume_init` here because the `MaybeUninit`
            // elements in the array do not require initialization.
            array: unsafe { MaybeUninit::uninit().assume_init() },
            initialized_len: 0,
        }
    }
}

impl<T, const N: usize> Deref for ArrayBuffer<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        let slice: &[MaybeUninit<T>] = &self.array[..self.initialized_len];
        // Safety: `slice` contains only the initialized part of the array.
        unsafe { MaybeUninit::slice_assume_init_ref(slice) }
    }
}

impl<T, const N: usize> DerefMut for ArrayBuffer<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        let slice: &mut [MaybeUninit<T>] = &mut self.array[..self.initialized_len];
        // Safety: `slice` contains only the initialized part of the array.
        unsafe { MaybeUninit::slice_assume_init_mut(slice) }
    }
}

#[cfg(feature = "alloc")]
mod vec_buffer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResultExt;
    use alloc::format;

    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    fn write_n<B: Buffer<u32>>(n: usize, buf: &mut B) -> Result<(), Option<usize>> {
        unsafe {
            buf.write_with_auto_resize(|data, size_in_bytes| {
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
                buf.write(|_, _| Status::UNSUPPORTED).status(),
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
    fn test_slice_buffer_maybe_uninit() {
        let slice: &mut [MaybeUninit<u32>] = &mut [
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
        ];
        let mut buf = SliceBuffer::from_maybe_uninit(slice);
        check_buffer(&mut buf, CappedAt3(true));
    }

    #[test]
    fn test_slice_buffer_from_ptr() {
        let slice = &mut [0, 0, 0];
        let mut buf = unsafe { SliceBuffer::from_ptr(slice.as_mut_ptr(), 3) };
        check_buffer(&mut buf, CappedAt3(true));
    }

    #[test]
    fn test_array_buffer() {
        let mut buf = ArrayBuffer::<u32, 3>::new();
        check_buffer(&mut buf, CappedAt3(true));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_vec_buffer() {
        let mut buf = Vec::new();
        check_buffer(&mut buf, CappedAt3(false));
    }
}
