// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
extern crate wio;
use wio::apc::{queue_current};
use wio::perf::{counter, frequency};
use wio::sleep::{sleep_alertable};
use wio::sleep::WakeReason::{CallbacksFired};
fn main() {
    let freq = frequency();
    queue_current(|| println!("1")).unwrap();
    queue_current(|| println!("2")).unwrap();
    queue_current(|| println!("3")).unwrap();
    let a = counter();
    assert_eq!(sleep_alertable(1000), CallbacksFired);
    let b = counter();
    println!("{}ms", (b - a) * 1_000 / freq);
}
