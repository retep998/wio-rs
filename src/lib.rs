// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
#![cfg(windows)]
#![allow(clippy::missing_safety_doc, clippy::len_without_is_empty)]
extern crate winapi;

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "log")]
macro_rules! log_if_feature {
    ($($args:tt)*) => {log::warn!($($args)*)};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "log"))]
macro_rules! log_if_feature {
    ($($args:tt)*) => {};
}

// pub mod apc;
pub mod bstr;
pub mod com;
pub mod console;
pub mod error;
pub mod handle;
pub mod mutex;
// pub mod perf;
// pub mod pipe;
// pub mod sleep;
// pub mod thread;
pub mod vsb;
pub mod wide;

pub use error::{Error, Result};
