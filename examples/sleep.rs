// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
extern crate wio;
use wio::apc::{queue_current};
use wio::perf::{counter, frequency};
use wio::sleep::{sleep_alertable};
fn main() {
    let freq = frequency();
    queue_current(|| println!("1")).unwrap();
    queue_current(|| println!("2")).unwrap();
    queue_current(|| println!("3")).unwrap();
    let a = counter();
    sleep_alertable(1000);
    let b = counter();
    println!("{}ms", (b - a) * 1_000 / freq);
}
