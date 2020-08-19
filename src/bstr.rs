// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use crate::wide::{FromWide, ToWide};
use std::{
    convert::TryInto,
    ffi::{OsStr, OsString},
    path::PathBuf,
    slice::from_raw_parts,
};
use winapi::{
    shared::wtypes::BSTR,
    um::oleauto::{
        SysAllocStringByteLen, SysAllocStringLen, SysFreeString, SysStringByteLen, SysStringLen,
    },
};
#[derive(Debug)]
pub struct BStr(BSTR);
impl BStr {
    pub unsafe fn from_raw(s: BSTR) -> BStr {
        BStr(s)
    }
    pub fn from_wide(s: &[u16]) -> BStr {
        unsafe { BStr(SysAllocStringLen(s.as_ptr(), s.len().try_into().unwrap())) }
    }
    pub fn from_bytes(s: &[u8]) -> BStr {
        unsafe {
            BStr(SysAllocStringByteLen(
                s.as_ptr().cast(),
                s.len().try_into().unwrap(),
            ))
        }
    }
    pub fn len(&self) -> usize {
        unsafe { SysStringLen(self.0) as usize }
    }
    pub fn byte_len(&self) -> usize {
        unsafe { SysStringByteLen(self.0) as usize }
    }
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
    pub fn as_ptr(&self) -> BSTR {
        self.0
    }
    pub fn as_wide(&self) -> &[u16] {
        if self.0.is_null() {
            &[]
        } else {
            unsafe { from_raw_parts(self.0, self.len()) }
        }
    }
    pub fn as_wide_null(&self) -> &[u16] {
        if self.0.is_null() {
            &[0]
        } else {
            unsafe { from_raw_parts(self.0, self.len() + 1) }
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        if self.0.is_null() {
            &[]
        } else {
            unsafe { from_raw_parts(self.0.cast(), self.byte_len()) }
        }
    }
    pub fn as_bytes_null(&self) -> &[u8] {
        if self.0.is_null() {
            &[0]
        } else {
            unsafe { from_raw_parts(self.0.cast(), self.byte_len() + 1) }
        }
    }
    pub fn to_string(&self) -> Option<String> {
        let os: OsString = self.into();
        os.into_string().ok()
    }
    pub fn to_string_lossy(&self) -> String {
        let os: OsString = self.into();
        os.into_string()
            .unwrap_or_else(|os| os.to_string_lossy().into_owned())
    }
}
impl Clone for BStr {
    fn clone(&self) -> BStr {
        BStr::from_wide(self.as_wide())
    }
}
impl Drop for BStr {
    fn drop(&mut self) {
        unsafe { SysFreeString(self.0) };
    }
}
impl<T> From<T> for BStr
where
    T: AsRef<OsStr>,
{
    fn from(s: T) -> BStr {
        BStr::from_wide(&s.to_wide())
    }
}
impl From<&BStr> for OsString {
    fn from(s: &BStr) -> OsString {
        OsString::from_wide(s.as_wide())
    }
}
impl From<&BStr> for PathBuf {
    fn from(s: &BStr) -> PathBuf {
        PathBuf::from_wide(s.as_wide())
    }
}
unsafe impl Send for BStr {}
unsafe impl Sync for BStr {}
