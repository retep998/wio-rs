// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>

use {Error, k32, w};
use std::boxed::{into_raw};

pub fn queue_apc_current<T>(func: T) -> Result<(), Error> where T: FnOnce() + 'static {
    extern "system" fn helper<T: FnOnce() + 'static>(thing: w::ULONG_PTR) {
        let func = unsafe { Box::from_raw(thing as *mut T) };
        func()
    }
    let thing = unsafe { into_raw(Box::new(func)) as w::ULONG_PTR };
    match unsafe { k32::QueueUserAPC(Some(helper::<T>), k32::GetCurrentThread(), thing) } {
        0 => {
            // If it fails we still need to deallocate the function
            unsafe { Box::from_raw(thing as *mut T) };
            Err(Error::last())
        },
        _ => Ok(()),
    }
}
