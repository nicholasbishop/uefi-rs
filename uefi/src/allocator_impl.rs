//! Allocator implementation for the global allocator.
//!
//! This is separate from `global_allocator.rs` so that we can run tests on it.

use crate::table::boot::MemoryType;
use crate::Result;
use core::alloc::Layout;
use core::{mem, ptr};

pub unsafe fn alloc<F>(layout: Layout, alloc_pool: F) -> *mut u8
where
    F: FnOnce(MemoryType, usize) -> Result<*mut u8>,
{
    let mem_ty = MemoryType::LOADER_DATA;
    let size = layout.size();
    let align = layout.align();

    // The memory returned by `EFI_BOOT_SERVICES.AllocatePool` is always 8-byte
    // aligned, so for any alignment up to 8 we can use that function directly.
    //
    // For higher alignments, we add the alignment as padding to the allocation
    // size so that we can return a pointer within the allocation with the
    // appropriate alignment. We also store the pointer to the full allocation
    // into the allocation itself, right before the part we return a pointer
    // to. This allows us to extract that pointer when `dealloc` is called so
    // that we can pass it to `EFI_BOOT_SERVICES.FreePool`.

    // Check that the pointer is of the expected size so that we can store it
    // within the allocation if necessary.
    debug_assert!(mem::size_of::<*mut u8>() <= 8);

    if align > 8 {
        // allocate more space for alignment
        let ptr = if let Ok(ptr) = alloc_pool(mem_ty, size + align) {
            ptr
        } else {
            return ptr::null_mut();
        };
        // calculate align offset
        let mut offset = ptr.align_offset(align);
        if offset == 0 {
            offset = align;
        }
        let return_ptr = ptr.add(offset);
        // store allocated pointer before the struct
        (return_ptr.cast::<*mut u8>()).sub(1).write(ptr);
        return_ptr
    } else {
        if let Ok(ptr) = alloc_pool(mem_ty, size) {
            debug_assert_eq!((ptr as usize) % 8, 0);
            ptr
        } else {
            ptr::null_mut()
        }
    }
}

pub unsafe fn dealloc<F>(mut ptr: *mut u8, layout: Layout, free_pool: F)
where
    F: FnOnce(*mut u8) -> Result,
{
    if layout.align() > 8 {
        ptr = (ptr as *const *mut u8).sub(1).read();
    }
    free_pool(ptr).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn alloc_fill_free(layout: Layout) {
        let mut inner_layout = None;

        let ptr = alloc(layout, |_, size| {
            inner_layout = Some(Layout::from_size_align(size, 8).unwrap());
            Ok(alloc::alloc::alloc(inner_layout.unwrap()))
        });

        ptr.write_bytes(0, layout.size());

        dealloc(ptr, layout, |ptr| {
            alloc::alloc::dealloc(ptr, inner_layout.unwrap());
            Ok(())
        });
    }

    #[test]
    fn test_alloc() {
        unsafe {
            alloc_fill_free(Layout::from_size_align(128, 4).unwrap());
            alloc_fill_free(Layout::from_size_align(128, 16).unwrap());
        }
    }
}
