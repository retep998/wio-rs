// Copyright © 2016, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
#![cfg(windows)]
extern crate winapi as w;
extern crate kernel32 as k32;

pub mod apc;
pub mod com;
pub mod console;
pub mod handle;
pub mod perf;
pub mod pipe;
pub mod sleep;
pub mod thread;
pub mod wide;

use std::io::{Error};

pub type IoResult<T> = Result<T, Error>;

fn last_error<T>() -> IoResult<T> {
    Err(Error::last_os_error())
}
