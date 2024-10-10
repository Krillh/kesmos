
#![allow(dead_code)]

const C_DEBUG_LEVEL: u8 = 0;
const E_DEBUG_LEVEL: u8 = 0;

mod expr;
mod expr2;
mod perf_test;
mod parse;
mod convert;
mod render;

use expr::{Context, f::*};

fn main() {
    //render::render();
    //parse_test();
    //perf_test::sin_cos_plane();
    //perf_test::multithread();
    // println!();
    // parse_test();
}

fn parse_test() {
    let f = std::fs::read_to_string("test.txt").unwrap();

    let p = parse::str_parse(&f);

    let c = convert::convert(p).simplify(true);

    let _ = std::fs::write("ctx.txt", format!("{c:#?}"));
    
    let o = c.evaluate_with_x("der", -3.0..=3.0, 500, true).unwrap();
    let mut o_s = String::new();
    o.iter().for_each(|(i,ot)| {
        o_s.push('\n'); o_s.push_str(&expr::fmt_1((*i,ot.clone().unwrap())));
    });

    let _ = std::fs::write("output.txt", format!("{}", o_s));
}

#[allow(dead_code)]
fn wavelet() {
    let c = Context::new()
    .func("gaus", vec!["t", "f", "n"], 
        *pow(var("e"),
            div(
            mul(
                mul(sq(var("t")), num(-2.0)),
                mul(var("pi"), var("f"))),
            mul(
                num(2.0), 
                var("n")))))
    .func("spiral", vec!["t", "f", "n"],
        *pow(var("e"), mul(
            mul(
                num(2.0),
                mul(
                    var("f"),
                    mul(var("i"), var("pi")))),
            var("t")
        )))
    .func("wave", vec!["t", "f", "n"],
        *mul(
            func("spiral", vec![var("t"), var("f"), var("n")]),
            func("gaus", vec![var("t"), var("f"), var("n")])
        ))
    .var("out", *func("wave", vec![var("x"), num(3.0), num(9.0)]))
    .var("tst", *add(num(0.0), var("i")))
    .simplify(true);


    let _ = std::fs::write("context", format!("{c:#?}"));

    
    let o = c.evaluate_with_x("tst", -20.0..=20.0, 8073, true).unwrap();
    let mut o_s = String::new();
    o.iter().for_each(|(i,ot)| {
        o_s.push('\n'); o_s.push_str(&expr::fmt_1((*i,ot.clone().unwrap())));
    });

    let _ = std::fs::write("output", format!("{}", o_s));
}