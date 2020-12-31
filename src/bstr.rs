// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use crate::wide::{FromWide, ToWide};
use std::{
    alloc::{handle_alloc_error, Layout},
    ptr::{self, NonNull},
    convert::TryInto,
    ffi::{OsStr, OsString},
    path::PathBuf,
    slice::from_raw_parts,
};
use winapi::{
    shared::{
        wtypes::BSTR,
        winerror::HRESULT,
    },
    um::{
        oleauto::{SysAllocStringByteLen, SysAllocStringLen, SysFreeString, SysStringByteLen, SysStringLen},
        winnt::WCHAR,
    }
};
#[derive(Debug)]
pub struct BStr(NonNull<WCHAR>);
impl BStr {
    pub unsafe fn new(s: BSTR) -> Option<BStr> {
        NonNull::new(s).map(BStr)
    }
    pub unsafe fn from_raw(s: BSTR) -> BStr {
        BStr(NonNull::new(s).expect("ptr should not be null"))
    }
    pub unsafe fn from_fn<F>(fun: F) -> Result<BStr, HRESULT>
    where
        F: FnOnce(&mut BSTR) -> HRESULT
    {
        let mut ptr: BSTR = ptr::null_mut();
        let res = fun(&mut ptr);
        let bstr = BStr::new(ptr);
        match res {
            0 => Ok(bstr.expect("fun must set bstr to a value")),
            res => {
                if bstr.is_some() {
                    log_if_feature!("BStr::from_fn had an initialized BSTR pointer despite the function returning an error");
                }
                Err(res)
            }
        }
    }
    pub fn from_wide(s: &[u16]) -> BStr {
        unsafe {
            let ptr = SysAllocStringLen(s.as_ptr(), s.len().try_into().unwrap());
            if ptr.is_null() {
                handle_alloc_error(Layout::array::<u16>(s.len()).unwrap())
            }
            BStr(NonNull::new_unchecked(ptr))
        }
    }
    pub fn from_bytes(s: &[u8]) -> BStr {
        unsafe {
            let ptr = SysAllocStringByteLen(s.as_ptr().cast(), s.len().try_into().unwrap());
            if ptr.is_null() {
                handle_alloc_error(Layout::array::<u8>(s.len()).unwrap())
            }
            BStr(NonNull::new_unchecked(ptr))
        }
    }
    pub fn len(&self) -> usize {
        unsafe { SysStringLen(self.0.as_ptr()) as usize }
    }
    pub fn byte_len(&self) -> usize {
        unsafe { SysStringByteLen(self.0.as_ptr()) as usize }
    }
    pub fn as_ptr(&self) -> BSTR {
        self.0.as_ptr()
    }
    pub fn as_wide(&self) -> &[u16] {
        unsafe { from_raw_parts(self.0.as_ptr(), self.len()) }
    }
    pub fn as_wide_null(&self) -> &[u16] {
        unsafe { from_raw_parts(self.0.as_ptr(), self.len() + 1) }
    }
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts(self.0.as_ptr().cast(), self.byte_len()) }
    }
    pub fn as_bytes_null(&self) -> &[u8] {
        // TODO: BECAUSE CHARS ARE WCHARS, SHOULD THIS BE +2 INSTEAD OF +1?
        unsafe { from_raw_parts(self.0.as_ptr().cast(), self.byte_len() + 1) }
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
        unsafe { SysFreeString(self.0.as_ptr()) };
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
