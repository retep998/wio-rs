// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use std::{
    alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc, Layout},
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::{self, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
};
/// This is a smart pointer type for holding FFI types whose size varies.
/// Most commonly this is with an array member as the last field whose size is specified
/// by either another field, or an external source of information.
pub struct VariableSizedBox<T> {
    size: usize,
    data: NonNull<T>,
    pd: PhantomData<T>,
}
impl<T> VariableSizedBox<T> {
    /// The size is specified in bytes. The data is uninitialized.
    pub fn new(size: usize) -> VariableSizedBox<T> {
        if size == 0 {
            return VariableSizedBox::default();
        }
        let layout = Layout::from_size_align(size, align_of::<T>()).unwrap();
        if let Some(data) = NonNull::new(unsafe { alloc(layout) }) {
            VariableSizedBox {
                size,
                data: data.cast(),
                pd: PhantomData,
            }
        } else {
            handle_alloc_error(layout)
        }
    }
    /// The size is specified in bytes. The data is zeroed.
    pub fn zeroed(size: usize) -> VariableSizedBox<T> {
        if size == 0 {
            return VariableSizedBox::default();
        }
        let layout = Layout::from_size_align(size, align_of::<T>()).unwrap();
        if let Some(data) = NonNull::new(unsafe { alloc_zeroed(layout) }) {
            VariableSizedBox {
                size,
                data: data.cast(),
                pd: PhantomData,
            }
        } else {
            handle_alloc_error(layout)
        }
    }
    /// Use this to get a pointer to pass to FFI functions.
    pub fn as_ptr(&self) -> *const T {
        if self.size == 0 {
            ptr::null()
        } else {
            self.data.as_ptr()
        }
    }
    /// Use this to get a pointer to pass to FFI functions.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        if self.size == 0 {
            ptr::null_mut()
        } else {
            self.data.as_ptr()
        }
    }
    /// This is used to more safely access the fixed size fields.
    /// # Safety
    /// The current data must be valid for an instance of `T`.
    pub unsafe fn as_ref(&self) -> &T {
        self.data.as_ref()
    }
    /// This is used to more safely access the fixed size fields.
    /// # Safety
    /// The current data must be valid for an instance of `T`.
    pub unsafe fn as_mut_ref(&mut self) -> &mut T {
        self.data.as_mut()
    }
    /// The size is specified in bytes.
    /// If this grows the allocation, the extra bytes will be uninitialized.
    /// I wish I could provide a zeroed alternative but Rust's stable allocators are lacking.
    pub fn resize(&mut self, size: usize) {
        if self.size == 0 || size == 0 {
            *self = VariableSizedBox::new(size);
            return;
        }
        let layout = Layout::from_size_align(self.size, align_of::<T>()).unwrap();
        if let Some(data) = NonNull::new(unsafe { realloc(self.as_mut_ptr().cast(), layout, size) })
        {
            self.data = data.cast();
            self.size = size;
        } else {
            handle_alloc_error(layout)
        }
    }
    /// The length of the allocation specified in bytes.
    pub fn len(&self) -> usize {
        self.size
    }
    /// Given a pointer to a specific field, upgrades the provenance of the pointer to the entire
    /// allocation to work around stacked borrows.
    /// # Safety
    /// `o` must be a valid pointer within the allocation contained by this box.
    pub unsafe fn sanitize_ptr<U>(&self, o: *const U) -> *const U {
        let offset = o as usize - self.as_ptr() as usize;
        (self.as_ptr() as *const u8).add(offset).cast()
    }
    /// Given a pointer to a specific field, upgrades the provenance of the pointer to the entire
    /// allocation to work around stacked borrows.
    /// # Safety
    /// `o` must be a valid pointer within the allocation contained by this box.
    pub unsafe fn sanitize_mut_ptr<U>(&mut self, o: *mut U) -> *mut U {
        let offset = o as usize - self.as_ptr() as usize;
        (self.as_mut_ptr() as *mut u8).add(offset).cast()
    }
    /// Given a pointer to a variable sized array field and the length of the array in elements,
    /// returns a slice to the entire variable sized array.
    /// # Safety
    /// The slice as specified by `o` and `count` must be entirely within the allocation
    /// contained by this box, and the data must be valid for the specified type.
    pub unsafe fn slice_from_count<U>(&self, o: *const U, count: usize) -> &[U] {
        let ptr = self.sanitize_ptr(o);
        assert!(ptr >= self.as_ptr().cast());
        assert!(count.saturating_mul(size_of::<U>()) <= self.size);
        assert!(ptr.wrapping_add(count) <= self.as_ptr().cast::<u8>().add(self.size).cast());
        from_raw_parts(ptr, count)
    }
    /// Given a pointer to a variable sized array field and the length of the array in elements,
    /// returns a mutable slice to the entire variable sized array.
    /// # Safety
    /// The slice as specified by `o` and `count` must be entirely within the allocation
    /// contained by this box, and the data must be valid for the specified type.
    pub unsafe fn slice_from_count_mut<U>(&mut self, o: *mut U, count: usize) -> &mut [U] {
        let ptr = self.sanitize_mut_ptr(o);
        assert!(ptr >= self.as_mut_ptr().cast());
        assert!(count.saturating_mul(size_of::<U>()) <= self.size);
        assert!(ptr.wrapping_add(count) <= self.as_mut_ptr().cast::<u8>().add(self.size).cast());
        from_raw_parts_mut(ptr, count)
    }
    /// Given a pointer to a variable sized array field and the length of the array in bytes,
    /// returns a slice to the entire variable sized array.
    /// # Safety
    /// The slice as specified by `o` and `bytes` must be entirely within the allocation
    /// contained by this box, and the data must be valid for the specified type.
    pub unsafe fn slice_from_bytes<U>(&self, o: *const U, bytes: usize) -> &[U] {
        let count = bytes / size_of::<U>();
        self.slice_from_count(o, count)
    }
    /// Given a pointer to a variable sized array field and the length of the array in bytes,
    /// returns a mutable slice to the entire variable sized array.
    /// # Safety
    /// The slice as specified by `o` and `bytes` must be entirely within the allocation
    /// contained by this box, and the data must be valid for the specified type.
    pub unsafe fn slice_from_bytes_mut<U>(&mut self, o: *mut U, bytes: usize) -> &mut [U] {
        let count = bytes / size_of::<U>();
        self.slice_from_count_mut(o, count)
    }
    /// Given a pointer to a variable sized array field and the size of the entire struct in bytes
    /// including the size of the array, returns a slice to the entire variable sized array.
    /// # Safety
    /// The slice as specified by `o` and `total_bytes` must be entirely within the allocation
    /// contained by this box, and the data must be valid for the specified type.
    pub unsafe fn slice_from_total_bytes<U>(&self, o: *const U, total_bytes: usize) -> &[U] {
        let bytes = total_bytes - (o as usize - self.as_ptr() as usize);
        self.slice_from_bytes(o, bytes)
    }
    /// Given a pointer to a variable sized array field and the size of the entire struct in bytes
    /// including the size of the array, returns a mutable slice to the entire variable sized
    /// array.
    /// # Safety
    /// The slice as specified by `o` and `total_bytes` must be entirely within the allocation
    /// contained by this box, and the data must be valid for the specified type.
    pub unsafe fn slice_from_total_bytes_mut<U>(
        &mut self,
        o: *mut U,
        total_bytes: usize,
    ) -> &mut [U] {
        let bytes = total_bytes - (o as usize - self.as_ptr() as usize);
        self.slice_from_bytes_mut(o, bytes)
    }
}
impl<T> Drop for VariableSizedBox<T> {
    fn drop(&mut self) {
        if self.size == 0 {
            return;
        }
        let layout = Layout::from_size_align(self.size, align_of::<T>()).unwrap();
        unsafe { dealloc(self.as_mut_ptr().cast(), layout) }
    }
}
impl<T> Default for VariableSizedBox<T> {
    fn default() -> Self {
        VariableSizedBox {
            size: 0,
            data: NonNull::dangling(),
            pd: PhantomData,
        }
    }
}
