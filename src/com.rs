// Copyright © 2016, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {w};
use std::ops::{Deref, DerefMut};

pub struct ComPtr<T>(*mut T);
impl<T> ComPtr<T> {
    /// Creates a `ComPtr` to wrap a raw pointer.
    /// It takes ownership over the pointer which means it does __not__ call `AddRef`.
    /// `T` __must__ be a COM interface that inherits from `IUnknown`.
    pub unsafe fn new(ptr: *mut T) -> ComPtr<T> { ComPtr(ptr) }
    /// Casts up the inheritance chain
    pub fn up<U>(&self) -> ComPtr<U> where T: Deref<Target=U> {
        unimplemented!()
    }
    /// Make sure you know what you're doing with this function
    pub unsafe fn as_mut(&self) -> &mut T {
        &mut*self.0
    }
    fn as_unknown(&self) -> &mut w::IUnknown {
        unsafe { &mut *(self.0 as *mut w::IUnknown) }
    }
}
impl<T> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.0 }
    }
}
impl<T> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut*self.0 }
    }
}
impl<T> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe { 
            self.as_unknown().AddRef();
            ComPtr::new(self.0)
        }
    }
}
impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe { self.as_unknown().Release(); }
    }
}
impl<T> PartialEq<ComPtr<T>> for ComPtr<T> {
    fn eq(&self, other: &ComPtr<T>) -> bool {
        self.0 == other.0
    }
}
