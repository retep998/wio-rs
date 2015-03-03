// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>

use {Error, k32, w};

pub struct File {
    handle: w::HANDLE,
}
impl Drop for File {
    fn drop(&mut self) {
        let err = unsafe { k32::CloseHandle(self.handle) };
        assert!(err != 0, "{}", Error::last());
    }
}
#[cfg(test)]
mod test {
    use {w};
    use file::{File};
    #[test] #[should_fail]
    fn test_file_drop() {
        drop(File { handle: w::INVALID_HANDLE_VALUE });
    }
}
