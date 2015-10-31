// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {k32};

pub fn frequency() -> i64 {
    let mut freq = 0;
    unsafe { k32::QueryPerformanceFrequency(&mut freq) };
    freq
}
pub fn counter() -> i64 {
    let mut count = 0;
    unsafe { k32::QueryPerformanceCounter(&mut count) };
    count
}
