// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
extern crate wio;
use wio::console::{ScreenBuffer};
use wio::sleep::{sleep};
fn main() {
    let buf = ScreenBuffer::from_stdin().unwrap();
    loop {
        let input = buf.read_input().unwrap();
        for i in input {
            println!("{:?}", i);
        }
    }
}
