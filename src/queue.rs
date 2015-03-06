// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>

use {Error, w, k32};
use std::boxed::{into_raw};
use std::marker::{PhantomData};
use std::ptr::{null_mut};

pub struct Queue<T> where T: Send + 'static {
    handle: w::HANDLE,
    phantom: PhantomData<T>,
}
impl<T> Queue<T> where T: Send + 'static {
    /// Pass 0 for concurrency to use the default which is the number of cpu cores
    pub fn new(concurrency: u32) -> Result<Queue<T>, Error> {
        let handle = unsafe {
            k32::CreateIoCompletionPort(w::INVALID_HANDLE_VALUE, null_mut(), 0, concurrency)
        };
        if handle == w::INVALID_HANDLE_VALUE { Err(Error::last()) }
        else { Ok(Queue { handle: handle, phantom: PhantomData }) }
    }
    pub fn send(&self, val: Box<T>) -> Result<(), Error> {
        match unsafe {
            k32::PostQueuedCompletionStatus(
                self.handle, 0, 0, into_raw(val) as w::LPOVERLAPPED,
            )
        } {
            0 => Err(Error::last()),
            _ => Ok(()),
        }
    }
    pub fn recv(&self) -> Result<Box<T>, Error> {
        let mut num = 0;
        let mut key = 0;
        let mut over = null_mut();
        match unsafe {
            k32::GetQueuedCompletionStatus(
                self.handle, &mut num as w::LPDWORD, &mut key as w::PULONG_PTR,
                &mut over as *mut w::LPOVERLAPPED, w::INFINITE,
            )
        } {
            0 => Err(Error::last()),
            _ => Ok(unsafe { Box::from_raw(over as *mut T) }),
        }
    }
}
#[unsafe_destructor]
impl<T> Drop for Queue<T> where T: Send + 'static {
    fn drop(&mut self) {
        let err = unsafe { k32::CloseHandle(self.handle) };
        assert!(err != 0, "{}", Error::last());
    }
}
unsafe impl<T> Send for Queue<T> {}
unsafe impl<T> Sync for Queue<T> {}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_queue() {
        let queue = Queue::new(0).unwrap();
        queue.send(Box::new(273)).unwrap();
        println!("{}", queue.recv().unwrap());
    }
}