// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::mem::forget;
use std::ops::Deref;
use std::ptr::{null_mut, NonNull};
use winapi::ctypes::c_void;
use winapi::um::unknwnbase::IUnknown;
use winapi::shared::guiddef::GUID;
use winapi::shared::winerror::HRESULT;
use winapi::Interface;

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "log")]
macro_rules! log_if_feature {
    ($($args:tt)*) => {log::warn!($($args)*)};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "log"))]
macro_rules! log_if_feature {
    ($($args:tt)*) => {};
}

/// Simplifies the common pattern of calling a function to initialize multiple `ComPtr`s.
///
/// This macro is a generalization of [`ComPtr::from_fn`][from_fn] to functions
/// that output multiple COM objects. It returns `Result<($(ComPtr<_>,)+), HRESULT>`, where the
/// `Ok` tuple contains the same number of `ComPtr` objects as the number of GUID/pointer pairs
/// passed to this macro.
///
/// See [`ComPtr::from_fn`][from_fn] for details on the exact semantics.
///
/// [from_fn]: crate::com::ComPtr::from_fn
#[macro_export]
macro_rules! com_ptr_from_fn {
    (
        |$(($guid:pat, $ptr:ident)),+ $(,)?| $init:expr
    ) => {{
        use winapi::Interface;
        use winapi::ctypes::c_void;
        use winapi::shared::guiddef::GUID;

        // hack to get the GUID through type inference
        struct GuidHack<T: Interface>(*mut T);
        impl<T: Interface> GuidHack<T> {
            fn guid(&self) -> GUID {
                T::uuidof()
            }
        }

        $(
            let mut $ptr = GuidHack(std::ptr::null_mut());
        )+

        let result: winapi::shared::winerror::HRESULT = {
            $(
                let $guid = &$ptr.guid();
                let $ptr = &mut *(&mut ($ptr.0) as *mut *mut _ as *mut *mut c_void);
            )+
            (|| $init)()
        };
        $(
            let $ptr = $crate::com::ComPtr::new($ptr.0);
        )+
        match result {
            0 => Ok(($(
                $ptr.expect(concat!("`", stringify!($expr), "` must set `", stringify!($ptr), "` to a value")),
            )+)),
            res => {
                $(if $ptr.is_some() {
                    $crate::log_if_feature!("ComPtr::from_fn had an initialized COM pointer despite the function returning an error");
                })+
                Err(res)
            }
        }
    }};
}

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
    /// Simplifies the common pattern of calling a function to initialize a `ComPtr`.
    ///
    /// `fun` gets passed `T`'s `GUID` and a mutable reference to a null pointer. If `fun` returns
    /// `S_OK`, it _must_ initialize the pointer to a non-null value.
    ///
    /// If `fun` *doesn't* return `S_OK` but still initializes the pointer, this function will
    /// assume that the pointer was initialized to a valid COM object and will call `Release` on
    /// it. If the `log` feature is enabled, it will emit a warning when that happens.
    ///
    /// May leak the COM pointer if the function panics after initializing the pointer.
    ///
    /// If you're calling a COM function that generates multiple COM objects, use the
    /// [`com_ptr_from_fn!`](../macro.com_ptr_from_fn.html) macro.
    pub unsafe fn from_fn<F>(fun: F) -> Result<ComPtr<T>, HRESULT>
    where
        T: Interface,
        F: FnOnce(&GUID, &mut *mut c_void) -> HRESULT
    {
        match com_ptr_from_fn!(|(guid, ptr)| fun(guid, ptr)) {
            Ok((p,)) => Ok(p),
            Err(e) => Err(e),
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
