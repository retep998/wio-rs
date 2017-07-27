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

#[derive(Clone, Copy, Debug)]
pub struct Error(w::DWORD);
impl Error {
    pub fn code(&self) -> u32 { self.0 }
}

pub type Result<T> = std::result::Result<T, Error>;

fn last_error<T>() -> Result<T> {
    Err(Error(unsafe { k32::GetLastError() }))
}
