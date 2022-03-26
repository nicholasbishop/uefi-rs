// Everything in this module is gated behind the `exts` feature.

use super::*;
use alloc_api::vec::Vec;
use core::fmt::{self, Debug, Formatter};

fn vec_write_impl<T, F>(
    v: &mut Vec<T>,
    mut f: F,
    max_len: Option<usize>,
) -> Result<(), Option<usize>>
where
    F: FnMut(*mut T, &mut usize) -> Status,
{
    let mut required_len = 0;
    match write_impl(&mut f, v.as_mut_ptr(), v.len(), &mut required_len) {
        // The existing vec was big enough.
        Ok(()) => {
            v.shrink_to(required_len);
            Ok(())
        }

        // Handle `BUFFER_TOO_SMALL` by resizing the vec.
        Err(err) if err.status() == Status::BUFFER_TOO_SMALL => {
            // OK to unwrap since `write_impl` fills in the error data.
            required_len = err.data().unwrap();

            // Return `BUFFER_TOO_SMALL` if the required length
            // exceeds the maximum length.
            if let Some(max_len) = max_len {
                if required_len > max_len {
                    return Err(Error::new(Status::BUFFER_TOO_SMALL, Some(required_len)));
                }
            }

            // Drop all current elements. This sets the length to zero but
            // does not affect the current allocation.
            v.truncate(0);

            // Reserve the nececessary number of elements. The input length
            // is relative to the vec's `len()`, which we know is zero.
            v.reserve_exact(required_len);

            // Get the uninitialized spare capacity (which is the whole
            // capacity in this case).
            let buf: &mut [MaybeUninit<T>] = v.spare_capacity_mut();
            debug_assert_eq!(buf.len(), required_len);

            let r = write_impl(
                &mut f,
                MaybeUninit::slice_as_mut_ptr(buf),
                buf.len(),
                &mut required_len,
            );

            // On success, mark the returned number of elements as initialized.
            if r.is_ok() {
                unsafe {
                    v.set_len(required_len);
                }
            }

            r
        }

        // Propagate all other errors.
        Err(err) => Err(err),
    }
}

impl<T> Buffer<T> for Vec<T> {
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        vec_write_impl(self, f, None)
    }
}

pub struct VecWithMaxLen<T> {
    vec: Vec<T>,
    max_len: usize,
}

impl<T> Buffer<T> for VecWithMaxLen<T> {
    unsafe fn write<F>(&mut self, f: F) -> Result<(), Option<usize>>
    where
        F: FnMut(*mut T, &mut usize) -> Status,
    {
        vec_write_impl(&mut self.vec, f, Some(self.max_len))
    }
}

impl<T: Debug> Debug for VecWithMaxLen<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Deref for VecWithMaxLen<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec
    }
}

impl<T> DerefMut for VecWithMaxLen<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec
    }
}

impl<T> VecWithMaxLen<T> {
    pub fn new(max_len: usize) -> Self {
        Self {
            vec: Vec::new(),
            max_len,
        }
    }
}
