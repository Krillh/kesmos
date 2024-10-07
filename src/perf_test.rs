
#![allow(dead_code)]

use std::{thread, time::Instant};


use crate::{convert, expr::{self, f::*, Context, Expr, Term}, parse};





//#[test]
pub fn sin_cos_plane() {

    let x_steps = 10000;
    let y_steps = 10000;

    let start = Instant::now();

    let c = Context::new()
    .var("foo", 
    Expr::Mul(
        Box::new(Expr::Sin(var("x"))), 
        Box::new(Expr::Cos(var("y")))))
    .simplify(true);

    let construction_time = start.elapsed();

    let mut results = Vec::new();

    let o = c.evaluate_with_xy("foo", 0.0..=5.0, x_steps, 0.0..=5.0, y_steps);
    for i in o.unwrap() {
        results.push((i.0, i.1, i.2.unwrap()));
    }

    let results_time = start.elapsed() - construction_time;

    println!("constuction time: {:?}\ncalculation time: {:?}", construction_time, results_time);
    println!("time per point: {:?}", results_time / (x_steps * y_steps) as u32);
}

pub fn multithread() {
    let thread_count = 16;

    let lower_bound = -3.1;
    let upper_bound = 3.1;
    let steps = 500;
    let step_size = (upper_bound - lower_bound) / steps as f64;
    let chunk_size = (upper_bound - lower_bound) / thread_count as f64;
    println!("{step_size}");

    let f = std::fs::read_to_string("test.txt").unwrap();
    let p = parse::str_parse(&f);
    let c = convert::convert(p).simplify(true);

    let mut threads = Vec::new();
    for t in 0..thread_count {
        let t_c = c.clone();
        let count = steps / thread_count;
        let bound = (lower_bound + (chunk_size * t as f64))..=((lower_bound + (chunk_size * (t+1) as f64)));
        threads.push(thread::spawn(move || {
            let o = t_c.evaluate_with_x("out", bound, count, true);
            return o;
        }));
    }

    let timer = Instant::now();
    
    let o = threads.into_iter().map(|t| t.join().unwrap().unwrap()).collect::<Vec<Vec<(f64, Option<Term>)>>>().concat();
    
    println!("\ncalculated {} points in {:?}", thread_count * (steps / thread_count), timer.elapsed());
    
    let timer = Instant::now();

    let mut o_s = String::new();
    o.iter().for_each(|(i,ot)| {
        o_s.push('\n'); o_s.push_str(&expr::fmt_1((*i,ot.clone().unwrap())));
    });

    println!("formatted in {:?}", timer.elapsed());

    let timer = Instant::now();

    let _ = std::fs::write("output.txt", format!("{}", o_s));

    println!("output to \"output.txt\" in {:?}", timer.elapsed());

}