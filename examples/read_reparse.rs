// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
#![feature(io, path)]
extern crate wio;
use std::io::{BufRead, stdin};
use std::path::{Path};
use wio::reparse::{reparse_read_path};
fn main() {
    let cin = stdin();
    let mut cin = cin.lock();
    let mut line = String::new();
    cin.read_line(&mut line).unwrap();
    let path = Path::new(line.trim());
    let res = reparse_read_path(&path);
    println!("{:?}", res);
}
