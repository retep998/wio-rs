// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {k32, w};

pub fn sleep(ms: u32) {
    unsafe { k32::Sleep(ms) }
}
#[derive(Debug, Eq, PartialEq)]
pub enum WakeReason {
    TimedOut,
    CallbacksFired,
}
pub fn sleep_alertable(ms: u32) -> WakeReason {
    let ret = unsafe { k32::SleepEx(ms, w::TRUE) };
    match ret {
        0 => WakeReason::TimedOut,
        w::WAIT_IO_COMPLETION => WakeReason::CallbacksFired,
        _ => unreachable!("SleepEx returned weird value of {:?}", ret),
    }
}
