// Copyright © 2016-2018, Peter Atashian
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use std::mem::forget;
use std::ops::Deref;
use std::ptr::null_mut;
#[cfg(feature = "com_nonnull")]
use std::ptr::NonNull;
use winapi::Interface;
use winapi::um::unknwnbase::IUnknown;

#[cfg(not(feature = "com_nonnull"))]
/// ComPtr to wrap COM interfaces sanely
pub struct ComPtr<T>(*mut T) where T: Interface;
#[cfg(feature = "com_nonnull")]
/// ComPtr to wrap COM interfaces sanely
pub struct ComPtr<T>(NonNull<T>) where T: Interface;
impl<T> ComPtr<T> where T: Interface {
    /// Creates a `ComPtr` to wrap a raw pointer.
    /// It takes ownership over the pointer which means it does __not__ call `AddRef`.
    /// `T` __must__ be a COM interface that inherits from `IUnknown`.
    #[cfg(not(feature = "com_nonnull"))]
    pub unsafe fn from_raw(ptr: *mut T) -> ComPtr<T> {
        assert!(!ptr.is_null());
        ComPtr(ptr)
    }
    /// Creates a `ComPtr` to wrap a raw pointer.
    /// It takes ownership over the pointer which means it does __not__ call `AddRef`.
    /// `T` __must__ be a COM interface that inherits from `IUnknown`.
    #[cfg(feature = "com_nonnull")]
    pub unsafe fn from_raw(ptr: *mut T) -> ComPtr<T> {
        assert!(!ptr.is_null());
        ComPtr(NonNull::new_unchecked(ptr))
    }
    /// Casts up the inheritance chain
    pub fn up<U>(self) -> ComPtr<U> where T: Deref<Target=U>, U: Interface {
        unsafe { ComPtr::from_raw(self.into_raw() as *mut U) }
    }
    /// Extracts the raw pointer.
    /// You are now responsible for releasing it yourself.
    pub fn into_raw(self) -> *mut T {
        let p = self.as_raw();
        forget(self);
        p
    }
    /// For internal use only.
    fn as_unknown(&self) -> &IUnknown {
        unsafe { &*(self.as_raw() as *mut IUnknown) }
    }
    /// Performs QueryInterface fun.
    pub fn cast<U>(&self) -> Result<ComPtr<U>, i32> where U: Interface {
        let mut obj = null_mut();
        let err = unsafe { self.as_unknown().QueryInterface(&U::uuidof(), &mut obj) };
        if err < 0 { return Err(err); }
        Ok(unsafe { ComPtr::from_raw(obj as *mut U) })
    }
    /// Obtains the raw pointer without transferring ownership.
    /// Do __not__ release this pointer because it is still owned by the `ComPtr`.
    #[cfg(not(feature = "com_nonnull"))]
    pub fn as_raw(&self) -> *mut T {
        self.0
    }
    /// Obtains the raw pointer without transferring ownership.
    /// Do __not__ release this pointer because it is still owned by the `ComPtr`.
    #[cfg(feature = "com_nonnull")]
    pub fn as_raw(&self) -> *mut T {
        self.0.as_ptr()
    }
}
impl<T> Deref for ComPtr<T> where T: Interface {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.as_raw() }
    }
}
impl<T> Clone for ComPtr<T> where T: Interface {
    fn clone(&self) -> Self {
        unsafe {
            self.as_unknown().AddRef();
            ComPtr::from_raw(self.as_raw())
        }
    }
}
impl<T> Drop for ComPtr<T> where T: Interface {
    fn drop(&mut self) {
        unsafe { self.as_unknown().Release(); }
    }
}
impl<T> PartialEq<ComPtr<T>> for ComPtr<T> where T: Interface {
    fn eq(&self, other: &ComPtr<T>) -> bool {
        self.as_raw() == other.as_raw()
    }
}
