// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use std::{
    alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc, Layout},
    marker::PhantomData,
    mem::{align_of, size_of},
    slice::{from_raw_parts, from_raw_parts_mut},
};
pub struct VariableSizedBox<T> {
    size: usize,
    data: *mut T,
    pd: PhantomData<T>,
}
impl<T> VariableSizedBox<T> {
    pub fn new(size: usize) -> VariableSizedBox<T> {
        let layout = Layout::from_size_align(size, align_of::<T>()).unwrap();
        let data = unsafe { alloc(layout) };
        if data.is_null() {
            handle_alloc_error(layout)
        }
        VariableSizedBox {
            size,
            data: data.cast(),
            pd: PhantomData,
        }
    }
    pub fn zeroed(size: usize) -> VariableSizedBox<T> {
        let layout = Layout::from_size_align(size, align_of::<T>()).unwrap();
        let data = unsafe { alloc_zeroed(layout) };
        if data.is_null() {
            handle_alloc_error(layout)
        }
        VariableSizedBox {
            size,
            data: data.cast(),
            pd: PhantomData,
        }
    }
    pub fn as_ptr(&self) -> *const T {
        self.data
    }
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data
    }
    pub unsafe fn as_ref(&self) -> &T {
        &*self.data
    }
    pub unsafe fn as_mut_ref(&mut self) -> &mut T {
        &mut *self.data
    }
    pub fn resize(&mut self, size: usize) {
        let layout = Layout::from_size_align(self.size, align_of::<T>()).unwrap();
        let data = unsafe { realloc(self.data.cast(), layout, size) };
        if data.is_null() {
            handle_alloc_error(layout)
        }
        self.data = data.cast();
        self.size = size;
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub unsafe fn sanitize_ptr<U>(&self, o: *const U) -> *const U {
        let offset = o as isize - self.data as isize;
        (self.data as *const u8).offset(offset).cast()
    }
    pub unsafe fn sanitize_mut_ptr<U>(&mut self, o: *mut U) -> *mut U {
        let offset = o as isize - self.data as isize;
        (self.data as *mut u8).offset(offset).cast()
    }
    pub unsafe fn slice_from_count<U>(&self, o: *const U, count: usize) -> &[U] {
        let ptr = self.sanitize_ptr(o);
        assert!(ptr >= self.data.cast());
        assert!(count.saturating_mul(size_of::<U>()) <= self.size);
        assert!(ptr.wrapping_add(count) <= self.data.cast::<u8>().add(self.size).cast());
        from_raw_parts(ptr, count)
    }
    pub unsafe fn slice_from_count_mut<U>(&mut self, o: *mut U, count: usize) -> &mut [U] {
        let ptr = self.sanitize_mut_ptr(o);
        assert!(ptr >= self.data.cast());
        assert!(count.saturating_mul(size_of::<U>()) <= self.size);
        assert!(ptr.wrapping_add(count) <= self.data.cast::<u8>().add(self.size).cast());
        from_raw_parts_mut(ptr, count)
    }
    pub unsafe fn slice_from_bytes<U>(&self, o: *const U, bytes: usize) -> &[U] {
        let count = bytes / size_of::<U>();
        self.slice_from_count(o, count)
    }
    pub unsafe fn slice_from_bytes_mut<U>(&mut self, o: *mut U, bytes: usize) -> &mut [U] {
        let count = bytes / size_of::<U>();
        self.slice_from_count_mut(o, count)
    }
    pub unsafe fn slice_from_total_bytes<U>(&self, o: *const U, total_bytes: usize) -> &[U] {
        let bytes = total_bytes - (o as usize - self.data as usize);
        self.slice_from_bytes(o, bytes)
    }
    pub unsafe fn slice_from_total_bytes_mut<U>(
        &mut self,
        o: *mut U,
        total_bytes: usize,
    ) -> &mut [U] {
        let bytes = total_bytes - (o as usize - self.data as usize);
        self.slice_from_bytes_mut(o, bytes)
    }
}
impl<T> Drop for VariableSizedBox<T> {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, align_of::<T>()).unwrap();
        unsafe { dealloc(self.data.cast(), layout) }
    }
}
