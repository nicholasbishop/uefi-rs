use alloc::alloc;
use core::alloc::Layout;
use core::any::Any;
use core::{mem, ptr};

pub struct SharedAnyBox {
    ptr: *mut dyn Any,
    layout: Layout,
}

impl SharedAnyBox {
    pub fn new<T: 'static>(val: T) -> Self {
        let layout = Layout::for_value(&val);
        let ptr = unsafe {
            let ptr: *mut T = alloc::alloc(layout).cast();
            ptr.write(val);
            ptr
        };
        Self { ptr, layout }
    }

    pub fn as_mut_ptr(&mut self) -> *mut dyn Any {
        self.ptr
    }

    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        unsafe { (*self.ptr).downcast_ref() }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        unsafe { (*self.ptr).downcast_mut() }
    }
}

impl Drop for SharedAnyBox {
    fn drop(&mut self) {
        // TODO: is this right? should test
        unsafe {
            ptr::drop_in_place(self.ptr);
            alloc::dealloc(self.ptr.cast(), self.layout);
        }
    }
}

pub struct SharedBox<T: ?Sized> {
    ptr: *mut T,
    layout: Layout,
}

impl<T: ?Sized + ptr_meta::Pointee> SharedBox<T> {
    pub fn new(val: &T) -> Self {
        let layout = Layout::for_value(val);
        let ptr = unsafe {
            let alloc_ptr: *mut u8 = alloc::alloc(layout).cast();
            let out_ptr: *mut T =
                ptr_meta::from_raw_parts_mut(alloc_ptr.cast(), ptr_meta::metadata(val));
            // TODO: pretty sure this is wrong since `val` could have
            // uninitialized padding bytes, but I'm not sure what the right way
            // to do it is.
            let src_ptr: *const T = val;
            alloc_ptr.copy_from_nonoverlapping(src_ptr.cast(), mem::size_of_val(val));
            out_ptr
        };
        Self { ptr, layout }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        assert!(!self.ptr.is_null());
        self.ptr
    }
}

impl<T> Default for SharedBox<T> {
    fn default() -> Self {
        Self {
            ptr: ptr::null_mut(),
            layout: Layout::from_size_align(mem::size_of::<T>(), mem::align_of::<T>()).unwrap(),
        }
    }
}

impl<T: ?Sized> Drop for SharedBox<T> {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }

        // TODO: is this right? should test
        unsafe {
            ptr::drop_in_place(self.ptr);
            alloc::dealloc(self.ptr.cast(), self.layout);
        }
    }
}
