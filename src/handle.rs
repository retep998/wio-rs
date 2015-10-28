// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {k32, w};
use std::io::{Error};
use std::ops::{Deref};
use std::os::windows::io::{AsRawHandle};

pub struct Handle(w::HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        let err = unsafe { k32::CloseHandle(self.0) };
        assert!(err != 0, "{:?}", Error::last_os_error());
    }
}
impl Deref for Handle {
    type Target = w::HANDLE;
    fn deref(&self) -> &w::HANDLE { &self.0 }
}
impl AsRawHandle for Handle {
    fn as_raw_handle(&self) -> w::HANDLE {
        self.0
    }
}
