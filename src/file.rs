// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>

use {Error, k32, w};
use std::ffi::AsOsStr;
use std::path::Path;
use std::ptr::{null_mut};
use std::os::windows::prelude::*;

pub struct File {
    handle: w::HANDLE,
}
impl File {
    pub fn new(path: &Path, mode: OpenMode) -> Result<File, Error> {
        let name: Vec<_> = path.as_os_str().encode_wide().collect();
        let name = name.as_ptr();
        let access = w::GENERIC_READ | w::GENERIC_WRITE;
        let share = w::FILE_SHARE_READ | w::FILE_SHARE_WRITE | w::FILE_SHARE_DELETE;
        let security = null_mut();
        let mode = mode as w::DWORD;
        let flags = w::FILE_FLAG_OVERLAPPED;
        let template = null_mut();
        let handle = unsafe {
            k32::CreateFileW(name, access, share, security, mode, flags, template)
        };
        if handle == w::INVALID_HANDLE_VALUE { Err(Error::last()) }
        else { Ok(File { handle: handle }) }
    }
}
impl Drop for File {
    fn drop(&mut self) {
        let err = unsafe { k32::CloseHandle(self.handle) };
        assert!(err != 0, "{}", Error::last());
    }
}
#[repr(u32)]
pub enum OpenMode {
    CreateNew = w::CREATE_NEW,
    CreateAlways = w::CREATE_ALWAYS,
    OpenExisting = w::OPEN_EXISTING,
    OpenAlways = w::OPEN_ALWAYS,
    TruncateExisting = w::TRUNCATE_EXISTING,
}
#[cfg(test)]
mod test {
    use {w};
    use super::*;
    use std::path::Path;
    #[test] #[should_fail]
    fn test_file_drop() {
        drop(File { handle: w::INVALID_HANDLE_VALUE });
    }
    #[test]
    fn make_file_success() {
        let p = Path::new("foo.txt");
        File::new(&p, OpenMode::CreateAlways).unwrap();
        File::new(&p, OpenMode::CreateAlways).unwrap();
    }
    #[test] #[should_fail]
    fn make_file_failure() {
        let p = Path::new("bar.txt");
        File::new(&p, OpenMode::CreateNew).unwrap();
        File::new(&p, OpenMode::CreateNew).unwrap();
    }
}
