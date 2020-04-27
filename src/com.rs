// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::mem::forget;
use std::ops::Deref;
use std::ptr::{null_mut, NonNull};
use winapi::um::unknwnbase::IUnknown;
use winapi::Interface;

// ComPtr to wrap COM interfaces sanely
#[repr(transparent)]
pub struct ComPtr<T>(NonNull<T>);
impl<T> ComPtr<T> {
    /// Creates a `ComPtr` to wrap a raw pointer.
    /// It takes ownership over the pointer which means it does __not__ call `AddRef`.
    /// `T` __must__ be a COM interface that inherits from `IUnknown`.
    pub unsafe fn new(ptr: *mut T) -> Option<ComPtr<T>>
    where
        T: Interface,
    {
        NonNull::new(ptr).map(ComPtr)
    }
    /// Creates a `ComPtr` to wrap a raw pointer.
    /// It takes ownership over the pointer which means it does __not__ call `AddRef`.
    /// `T` __must__ be a COM interface that inherits from `IUnknown`.
    /// The raw pointer must not be null or this function will panic.
    pub unsafe fn from_raw(ptr: *mut T) -> ComPtr<T>
    where
        T: Interface,
    {
        ComPtr(NonNull::new(ptr).expect("ptr should not be null"))
    }
    /// Simplifies the common pattern of calling a function to initialize a ComPtr.
    /// May leak the COM pointer if the function panics after initializing the pointer.
    /// The pointer provided to the function starts as a null pointer.
    /// If the pointer is initialized to a non-null value, it will be interpreted as a valid COM
    /// pointer, even if the function returns an error in which case it will be released by
    /// `from_fn` and a warning logged if logging is enabled.
    pub unsafe fn from_fn<F, E>(fun: F) -> Result<Option<ComPtr<T>>, E>
    where
        T: Interface,
        F: FnOnce(&mut *mut T) -> Result<(), E>,
    {
        let mut ptr = null_mut();
        let res = fun(&mut ptr);
        let com = ComPtr::new(ptr);
        match res {
            Ok(()) => Ok(com),
            Err(err) => {
                if com.is_some() {
                    #[cfg(feature = "log")] log::warn!("ComPtr::from_fn had an initialized COM pointer despite the function returning an error")
                }
                Err(err)
            }
        }
    }
    /// Casts up the inheritance chain
    pub fn up<U>(self) -> ComPtr<U>
    where
        T: Deref<Target = U>,
        U: Interface,
    {
        unsafe { ComPtr::from_raw(self.into_raw() as *mut U) }
    }
    /// Extracts the raw pointer.
    /// You are now responsible for releasing it yourself.
    pub fn into_raw(self) -> *mut T {
        let p = self.0.as_ptr();
        forget(self);
        p
    }
    /// For internal use only.
    fn as_unknown(&self) -> &IUnknown {
        unsafe { &*(self.as_raw() as *mut IUnknown) }
    }
    /// Performs QueryInterface fun.
    pub fn cast<U>(&self) -> Result<ComPtr<U>, i32>
    where
        U: Interface,
    {
        let mut obj = null_mut();
        let err = unsafe { self.as_unknown().QueryInterface(&U::uuidof(), &mut obj) };
        if err < 0 {
            return Err(err);
        }
        Ok(unsafe { ComPtr::from_raw(obj as *mut U) })
    }
    /// Obtains the raw pointer without transferring ownership.
    /// Do __not__ release this pointer because it is still owned by the `ComPtr`.
    pub fn as_raw(&self) -> *mut T {
        self.0.as_ptr()
    }
}
impl<T> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.as_raw() }
    }
}
impl<T> Clone for ComPtr<T>
where
    T: Interface,
{
    fn clone(&self) -> Self {
        unsafe {
            self.as_unknown().AddRef();
            ComPtr::from_raw(self.as_raw())
        }
    }
}
impl<T> Debug for ComPtr<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{:?}", self.0)
    }
}
impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            self.as_unknown().Release();
        }
    }
}
impl<T> PartialEq<ComPtr<T>> for ComPtr<T>
where
    T: Interface,
{
    fn eq(&self, other: &ComPtr<T>) -> bool {
        self.0 == other.0
    }
}
