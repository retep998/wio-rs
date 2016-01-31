// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {IoResult, k32, last_error, w};
use std::io::{Error};
use std::ops::{Deref};
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle};
use std::ptr::{null_mut};

pub struct Handle(w::HANDLE);
impl Handle {
    pub fn close(self) -> IoResult<()> {
        match unsafe { k32::CloseHandle(self.into_raw_handle()) } {
            0 => last_error(),
            _ => Ok(()),
        }
    }
    pub unsafe fn duplicate_from(handle: w::HANDLE) -> IoResult<Handle> {
        let mut new_handle = null_mut();
        let res = k32::DuplicateHandle(
            k32::GetCurrentProcess(), handle, k32::GetCurrentProcess(),
            &mut new_handle, 0, w::FALSE, w::DUPLICATE_SAME_ACCESS,
        );
        match res {
            0 => last_error(),
            _ => Ok(Handle(new_handle)),
        }
    }
}
impl AsRawHandle for Handle {
    fn as_raw_handle(&self) -> w::HANDLE {
        self.0
    }
}
impl Deref for Handle {
    type Target = w::HANDLE;
    fn deref(&self) -> &w::HANDLE { &self.0 }
}
impl Drop for Handle {
    fn drop(&mut self) {
        let err = unsafe { k32::CloseHandle(self.0) };
        assert!(err != 0, "{:?}", Error::last_os_error());
    }
}
impl FromRawHandle for Handle {
    unsafe fn from_raw_handle(handle: w::HANDLE) -> Handle {
        Handle(handle)
    }
}
impl IntoRawHandle for Handle {
    fn into_raw_handle(self) -> w::HANDLE {
        self.0
    }
}
