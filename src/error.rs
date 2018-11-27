// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.

use libc::wcslen;
use std::error::Error as StdError;
use std::fmt;
use std::ffi::OsString;
use std::ptr;
use std::result;
use std::slice;
use std::os::windows::ffi::OsStringExt;
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::{self, FormatMessageW};
use winapi::um::winnt::{self, MAKELANGID, WCHAR};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Error(DWORD);

impl Error {
    pub fn code(&self) -> u32 {
        self.0
    }

    pub fn last<T>() -> Result<T> {
        Err(Error(unsafe { GetLastError() }))
    }

    #[must_use]
    pub fn system_message(&self) -> OsString {
        unsafe {
            let mut error_text: *mut WCHAR = ptr::null_mut();

            FormatMessageW(
                winbase::FORMAT_MESSAGE_FROM_SYSTEM
                    | winbase::FORMAT_MESSAGE_ALLOCATE_BUFFER
                    | winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
                ptr::null_mut(),
                self.code() as _,
                MAKELANGID(winnt::LANG_NEUTRAL, winnt::SUBLANG_DEFAULT) as _,
                &mut error_text as *mut _ as *mut _,
                0,
                ptr::null_mut(),
            );

            let wchars = if !error_text.is_null() {
                slice::from_raw_parts(error_text, wcslen(error_text))
            } else {
                &[]
            };

            let message = OsString::from_wide(wchars);

            if !error_text.is_null() {
                winbase::LocalFree(error_text as *mut _);
            }

            message
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = self.system_message().into_string().ok();
        f.pad(desc.as_ref().map(String::as_str).unwrap_or("Unknown error"))
    }
}

impl Into<u32> for Error {
    fn into(self) -> u32 {
        self.code()
    }
}

impl StdError for Error {}

pub type Result<T> = result::Result<T, Error>;
