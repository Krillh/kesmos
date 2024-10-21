
#![allow(dead_code)]

const C_DEBUG_LEVEL: u8 = 0;
const E_DEBUG_LEVEL: u8 = 0;

mod expr;
mod perf_test;
mod parse;
mod convert;
mod render;

use std::fs;

use expr::{Context, f::*};

fn main() {
    parse_test();
    
}

fn parse_test() {
    println!("reading file...");
    let f = std::fs::read_to_string("tst/test.txt").unwrap();
    println!("parsing file...");
    let p = parse::str_parse(&f);
    println!("converting tokens...");
    let c = convert::convert(p);
    fs::write("ctx.txt", format!("{c:#?}"));
    println!("simplifying...");
    let e = c.simplify_for_var("out");
    println!("writing to file...");
    fs::write("output.txt", format!("{e:#?}"));
}


