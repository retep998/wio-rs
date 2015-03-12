// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>

use {Error, Handle, k32, w};
use std::ffi::{AsOsStr, OsString};
use std::os::windows::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr::{null_mut};
use std::slice::{from_raw_parts};

#[derive(Debug)]
pub enum ReparsePoint {
    AbsoluteSymlink(PathBuf, PathBuf),
    RelativeSymlink(PathBuf, PathBuf),
    MountPoint(PathBuf, PathBuf),
    Other,
}

pub fn reparse_read_path(path: &Path) -> Result<ReparsePoint, Error> {
    let name: Vec<_> = path.as_os_str().encode_wide().chain(Some(0).into_iter()).collect();
    let handle = unsafe {
        k32::CreateFileW(
            name.as_ptr(), 0, w::FILE_SHARE_READ | w::FILE_SHARE_WRITE | w::FILE_SHARE_DELETE,
            null_mut(), w::OPEN_EXISTING, w::FILE_FLAG_OPEN_REPARSE_POINT, null_mut(),
        )
    };
    if handle == w::INVALID_HANDLE_VALUE { return Err(Error::last()) }
    let handle = Handle(handle);
    reparse_read_handle(*handle)
}
pub fn reparse_read_handle(handle: w::HANDLE) -> Result<ReparsePoint, Error> {
    #[repr(C)]
    struct ReparseHead {
        tag: u32,
        _length: u16,
        _reserved: u16,
        rest: (),
    }
    let mut buf = vec![0u8; 0x10000];
    let mut bytes = 0;
    if unsafe {
        k32::DeviceIoControl(
            handle, w::FSCTL_GET_REPARSE_POINT, null_mut(), 0, buf.as_mut_ptr() as w::LPVOID, buf.len() as w::DWORD, &mut bytes as w::LPDWORD, null_mut(),
        )
    } == 0 { return Err(Error::last()) }
    let head = unsafe { &*(buf.as_ptr() as *const ReparseHead) };
    match head.tag {
        w::IO_REPARSE_TAG_SYMLINK => {
            #[repr(C)]
            struct ReparseSymlink {
                substoff: u16,
                substlen: u16,
                printoff: u16,
                printlen: u16,
                flags: u32,
                pathbuf: (),
            }
            let reparse = unsafe { &*(&head.rest as *const _ as *const ReparseSymlink) };
            let path = &reparse.pathbuf as *const _ as *const u8;
            let subst = unsafe { path.offset(reparse.substoff as isize) as *const u16 };
            let subst = unsafe { from_raw_parts(subst, (reparse.substlen / 2) as usize) };
            let subst = PathBuf::new(&OsString::from_wide(subst));
            let print = unsafe { path.offset(reparse.printoff as isize) as *const u16 };
            let print = unsafe { from_raw_parts(print, (reparse.printlen / 2) as usize) };
            let print = PathBuf::new(&OsString::from_wide(print));
            if reparse.flags & 0x1 == 0 {
                Ok(ReparsePoint::AbsoluteSymlink(subst, print))
            } else {
                Ok(ReparsePoint::RelativeSymlink(subst, print))
            }
        },
        _ => Ok(ReparsePoint::Other),
    }
}
