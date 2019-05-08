// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.

use std::{
    fmt::{Debug, Error as FmtError, Formatter},
    marker::PhantomData,
    mem::size_of,
    ops::Deref,
    ptr::null_mut,
};
use winapi::{
    shared::{
        minwindef::FALSE,
        winerror::WAIT_TIMEOUT,
    },
    um::{
        errhandlingapi::GetLastError,
        minwinbase::SECURITY_ATTRIBUTES,
        synchapi::{CreateMutexW, OpenMutexW, ReleaseMutex, WaitForSingleObject},
        winbase::{INFINITE, WAIT_ABANDONED, WAIT_OBJECT_0},
        winnt::{HANDLE, SECURITY_DESCRIPTOR, SYNCHRONIZE},
    },
};
use error::Error;
use handle::Handle;
use security_attributes::SecurityAttributes;
use wide::ToWide;

pub struct SecurityAttributes(SECURITY_ATTRIBUTES);
impl SecurityAttributes {
    pub unsafe fn from_raw(sd: *mut SECURITY_DESCRIPTOR) -> SecurityAttributes {
        SecurityAttributes(SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd as *mut _,
            bInheritHandle: FALSE,
        })
    }
}

pub struct Mutex<T>(Handle, T);
impl<T> Mutex<T> {
    pub fn create(data: T, mut security_attributes: Option<SecurityAttributes>, name: &str) -> Result<Mutex<T>, InitError<T>> {
        unsafe {
            let handle = CreateMutexW(
                security_attributes.as_mut().map(|x| &mut x.0 as *mut _).unwrap_or(null_mut()),
                0,
                name.to_wide_null().as_ptr(),
            );
            if handle.is_null() {
                return Err(InitError { data: data, error: Error::last() });
            }
            Ok(Mutex(Handle::new(handle), data))
        }
    }
    pub fn open(data: T, name: &str) -> Result<Mutex<T>, InitError<T>> {
        unsafe {
            let handle = OpenMutexW(
                SYNCHRONIZE,
                FALSE,
                name.to_wide_null().as_ptr(),
            );
            if handle.is_null() {
                return Err(InitError { data: data, error: Error::last() });
            }
            Ok(Mutex(Handle::new(handle), data))
        }
    }
    /// The timeout is specified in milliseconds
    /// Specifying None for the timeout means to wait forever
    pub fn wait<'a>(&'a self, timeout: Option<u32>) -> Result<MutexGuard<'a, T>, WaitError<'a, T>> {
        unsafe {
            match WaitForSingleObject(*self.0, timeout.unwrap_or(INFINITE)) {
                WAIT_ABANDONED => Err(WaitError::Abandoned(AbandonedMutexGuard::new(self))),
                WAIT_OBJECT_0 => Ok(MutexGuard::new(self)),
                WAIT_TIMEOUT => Err(WaitError::Timeout),
                _ => Err(WaitError::Other(Error::last())),
            }
        }
    }
    pub fn try_clone(&self) -> Result<Mutex<T>, Error> where T: Clone {
        unsafe {
            let handle = Handle::duplicate_from(*self.0)?;
            Ok(Mutex(handle, self.1.clone()))
        }
    }
}
impl<T> Debug for Mutex<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self.wait(Some(0)) {
            Ok(guard) => {
                f.debug_struct("Mutex").field("handle", &*self.0)
                    .field("data", &*guard).finish()
            },
            Err(err) => {
                f.debug_struct("Mutex").field("handle", &*self.0)
                    .field("data", &err).finish()
            }
        }
    }
}
unsafe impl<T> Send for Mutex<T> where T: Send {}
unsafe impl<T> Sync for Mutex<T> where T: Sync {}

pub struct MutexGuard<'a, T>(&'a Mutex<T>, PhantomData<HANDLE>);
impl<'a, T> MutexGuard<'a, T> {
    unsafe fn new(mutex: &'a Mutex<T>) -> MutexGuard<'a, T> {
        MutexGuard(mutex, PhantomData)
    }
}
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            if ReleaseMutex(*(self.0).0) == 0 {
                let err = GetLastError();
                panic!("failed to call ReleaseMutex: {}", err);
            }
        }
    }
}
impl<'a, T> Debug for MutexGuard<'a, T> where T: Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("MutexGuard").field("handle", &*(self.0).0)
            .field("data", &(self.0).1).finish()
    }
}
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &(self.0).1
    }
}

pub struct AbandonedMutexGuard<'a, T>(&'a Mutex<T>, PhantomData<HANDLE>);
impl<'a, T> AbandonedMutexGuard<'a, T> {
    unsafe fn new(mutex: &'a Mutex<T>) -> AbandonedMutexGuard<'a, T> {
        AbandonedMutexGuard(mutex, PhantomData)
    }
    pub fn unabandon(self) -> MutexGuard<'a, T> {
        MutexGuard(self.0, self.1)
    }
}
impl<'a, T> Debug for AbandonedMutexGuard<'a, T> where T: Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str("<abandoned>")
    }
}
#[derive(Debug)]
pub struct InitError<T> {
    pub data: T,
    pub error: Error,
}
#[derive(Debug)]
pub enum WaitError<'a, T> {
    Timeout,
    Abandoned(AbandonedMutexGuard<'a, T>),
    Other(Error),
}
