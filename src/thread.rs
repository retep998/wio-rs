// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {Result, k32, last_error, w};
use handle::{Handle};
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle};
use std::thread::{JoinHandle};

pub struct Thread(Handle);
impl Thread {
    pub fn current() -> Result<Thread> {
        unsafe { Handle::duplicate_from(k32::GetCurrentThread()).map(Thread) }
    }
    /// Returns the old affinity mask on success
    pub fn set_affinity_mask(&self, mask: usize) -> Result<usize> {
        let res = unsafe {
            k32::SetThreadAffinityMask(*self.0, mask as w::ULONG_PTR)
        };
        match res {
            0 => last_error(),
            prev => Ok(prev as usize),
        }
    }
}
impl<T> From<JoinHandle<T>> for Thread {
    fn from(o: JoinHandle<T>) -> Thread {
        unsafe { Thread::from_raw_handle(o.into_raw_handle()) }
    }
}
impl<'a, T> From<&'a JoinHandle<T>> for Thread {
    fn from(o: &'a JoinHandle<T>) -> Thread {
        unsafe { Thread::from_raw_handle(o.as_raw_handle()) }
    }
}
impl AsRawHandle for Thread {
    fn as_raw_handle(&self) -> w::HANDLE {
        self.0.as_raw_handle()
    }
}
impl IntoRawHandle for Thread {
    fn into_raw_handle(self) -> w::HANDLE {
        self.0.into_raw_handle()
    }
}
impl FromRawHandle for Thread {
    unsafe fn from_raw_handle(handle: w::HANDLE) -> Thread {
        Thread(Handle::from_raw_handle(handle))
    }
}
