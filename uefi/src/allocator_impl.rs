//! Allocator implementation for the global allocator.
//!
//! This is separate from `global_allocator.rs` so that we can run tests on it.

use crate::table::boot::MemoryType;
use crate::Result;
use core::alloc::Layout;
use core::ptr;

pub unsafe fn alloc<F>(layout: Layout, alloc_pool: F) -> *mut u8
where
    F: FnOnce(MemoryType, usize) -> Result<*mut u8>,
{
    let mem_ty = MemoryType::LOADER_DATA;
    let size = layout.size();
    let align = layout.align();

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
        alloc_pool(mem_ty, size).unwrap_or(ptr::null_mut())
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
