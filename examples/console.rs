// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
extern crate wio;
use wio::console::{ScreenBuffer};
use wio::sleep::{sleep};
fn main() {
    let buf = ScreenBuffer::from_stdout().unwrap();
    buf.set_active();
    println!("Hello world!");
    println!("{:?}", buf.info());
    sleep(5000);
}
