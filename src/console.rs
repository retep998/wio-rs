// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {IoResult, k32, last_error, w};
use handle::{Handle};
use std::ptr::{null, null_mut};
use std::os::windows::io::{FromRawHandle};

pub struct Console(Handle);

impl Console {
    pub fn new() -> IoResult<Console> {
        let handle = unsafe { k32::CreateConsoleScreenBuffer(
            w::GENERIC_READ | w::GENERIC_WRITE,
            w::FILE_SHARE_READ | w::FILE_SHARE_WRITE,
            null(),
            w::CONSOLE_TEXTMODE_BUFFER,
            null_mut(),
        )};
        if handle == w::INVALID_HANDLE_VALUE { last_error() }
        else { Ok(Console(unsafe { Handle::from_raw_handle(handle) })) }
    }
    pub fn set_active(&self) -> IoResult<()> {
        let res = unsafe { k32::SetConsoleActiveScreenBuffer(*self.0) };
        if res == 0 { last_error() }
        else { Ok(()) }
    }
}

/// Allocates a console if the process does not already have a console.
pub fn alloc() -> IoResult<()> {
    match unsafe { k32::AllocConsole() } {
        0 => last_error(),
        _ => Ok(()),
    }
}
/// Detaches the process from its current console.
pub fn free() -> IoResult<()> {
    match unsafe { k32::FreeConsole() } {
        0 => last_error(),
        _ => Ok(()),
    }
}
/// Attaches the process to the console of the specified process.
/// Pass None to attach to the console of the parent process.
pub fn attach(processid: Option<u32>) -> IoResult<()> {
    match unsafe { k32::AttachConsole(processid.unwrap_or(-1i32 as u32)) } {
        0 => last_error(),
        _ => Ok(()),
    }
}
