// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {w};
use handle::{Handle};
use std::os::windows::io::{AsRawHandle};

pub struct Thread(Handle);
impl AsRawHandle for Thread {
    fn as_raw_handle(&self) -> w::HANDLE {
        self.0.as_raw_handle()
    }
}
