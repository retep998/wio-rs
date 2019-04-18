// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.

use std::ptr::null_mut;
use winapi::{
    shared::{
        minwindef::FALSE,
        winerror::WAIT_TIMEOUT,
    },
    um::{
        errhandlingapi::GetLastError,
        handleapi::{CloseHandle, DuplicateHandle},
        minwinbase::SECURITY_ATTRIBUTES,
        processthreadsapi::GetCurrentProcess,
        synchapi::{CreateMutexW, OpenMutexW, ReleaseMutex, WaitForSingleObject},
        winbase::{INFINITE, WAIT_ABANDONED, WAIT_OBJECT_0},
        winnt::{DUPLICATE_SAME_ACCESS, HANDLE, SYNCHRONIZE},
    },
};
use error::Error;
use wide::ToWide;

pub struct Mutex(HANDLE);
impl Mutex {
    pub fn create(security_attributes: Option<&SECURITY_ATTRIBUTES>, name: &str) -> Result<Mutex, Error> {
        unsafe {
            let handle = CreateMutexW(
                security_attributes.map(|x| x as *const _ as *mut _).unwrap_or(null_mut()),
                0,
                name.to_wide_null().as_ptr(),
            );
            if handle.is_null() {
                return Error::last();
            }
            Ok(Mutex(handle))
        }
    }
    pub fn open(name: &str) -> Result<Mutex, Error> {
        unsafe {
            let handle = OpenMutexW(
                SYNCHRONIZE,
                FALSE,
                name.to_wide_null().as_ptr(),
            );
            if handle.is_null() {
                return Error::last();
            }
            Ok(Mutex(handle))
        }
    }
    pub fn wait<'a>(&'a self) -> Result<Result<MutexGuard<'a>, MutexGuard<'a>>, Error> {
        unsafe {
            match WaitForSingleObject(self.0, INFINITE) {
                WAIT_ABANDONED => Ok(Err(MutexGuard(self, self.0))),
                WAIT_OBJECT_0 => Ok(Ok(MutexGuard(self, self.0))),
                _ => Error::last(),
            }
        }
    }
    pub fn wait_timeout<'a>(&'a self, timeout: u32) -> Result<Option<Result<MutexGuard<'a>, MutexGuard<'a>>>, Error> {
        unsafe {
            match WaitForSingleObject(self.0, timeout) {
                WAIT_ABANDONED => Ok(Some(Err(MutexGuard(self, self.0)))),
                WAIT_OBJECT_0 => Ok(Some(Ok(MutexGuard(self, self.0)))),
                WAIT_TIMEOUT => Ok(None),
                _ => Error::last(),
            }
        }
    }
    pub fn try_clone(&self) -> Result<Mutex, Error> {
        unsafe {
        let mut handle = null_mut();
        if DuplicateHandle(
            GetCurrentProcess(), self.0, GetCurrentProcess(),
            &mut handle, 0, FALSE, DUPLICATE_SAME_ACCESS,
        ) == 0 {
            return Error::last();
        }
        Ok(Mutex(handle))
        }
    }
}
impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe {
            if CloseHandle(self.0) == 0 {
                let err = GetLastError();
                panic!("failed to call CloseHandle: {}", err);
            }
        }
    }
}
unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}
pub struct MutexGuard<'a>(&'a Mutex, HANDLE);
impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            if ReleaseMutex(self.1) == 0 {
                let err = GetLastError();
                panic!("failed to call ReleaseMutex: {}", err);
            }
        }
    }
}
