// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.

use std::{
    marker::PhantomData,
    ptr::null_mut,
};
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
use handle::Handle;
use wide::ToWide;

pub struct Mutex(Handle);
impl Mutex {
    pub fn create(security_attributes: Option<&SECURITY_ATTRIBUTES>, name: &str) -> Result<Mutex, Error> {
        unsafe {
            let handle = CreateMutexW(
                security_attributes.map(|x| x as *const _ as *mut _).unwrap_or(null_mut()),
                0,
                name.to_wide_null().as_ptr(),
            );
            if handle.is_null() {
                return Err(Error::last());
            }
            Ok(Mutex(Handle::new(handle)))
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
                return Err(Error::last());
            }
            Ok(Mutex(Handle::new(handle)))
        }
    }
    /// The timeout is specified in milliseconds
    /// Specifying None for the timeout means to wait forever
    pub fn wait<'a>(&'a self, timeout: Option<u32>) -> Result<MutexGuard<'a>, WaitError<'a>> {
        unsafe {
            match WaitForSingleObject(*self.0, timeout.unwrap_or(INFINITE)) {
                WAIT_ABANDONED => Err(WaitError::Abandoned(MutexGuard::new(self))),
                WAIT_OBJECT_0 => Ok(MutexGuard::new(self)),
                WAIT_TIMEOUT => Err(WaitError::Timeout),
                _ => Err(WaitError::Other(Error::last())),
            }
        }
    }
    pub fn try_clone(&self) -> Result<Mutex, Error> {
        unsafe {
            let mut handle = null_mut();
            if DuplicateHandle(
                GetCurrentProcess(), *self.0, GetCurrentProcess(),
                &mut handle, 0, FALSE, DUPLICATE_SAME_ACCESS,
            ) == 0 {
                return Err(Error::last());
            }
            Ok(Mutex(Handle::new(handle)))
        }
    }
}
unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}
pub struct MutexGuard<'a>(HANDLE, PhantomData<&'a Mutex>);
impl<'a> MutexGuard<'a> {
    unsafe fn new(mutex: &'a Mutex) -> MutexGuard<'a> {
        MutexGuard(*mutex.0, PhantomData)
    }
}
impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            if ReleaseMutex(self.0) == 0 {
                let err = GetLastError();
                panic!("failed to call ReleaseMutex: {}", err);
            }
        }
    }
}
pub enum WaitError<'a> {
    Timeout,
    Abandoned(MutexGuard<'a>),
    Other(Error),
}
