

use std::ops::RangeInclusive;
use std::time::Instant;

use crate::{convert, expr, parse};
use kiss3d::nalgebra::Point3;
use kiss3d::window::Window;
use kiss3d::light::Light;


pub fn line(w: &mut Window, points: &Vec<(f64, f64, f64)>) {
    for i in 1..points.len() {
        let a = points[i-1];
        let b = points[i];
        let p_a = Point3::new(a.0 as f32, a.1 as f32, a.2 as f32);
        let p_b = Point3::new(b.0 as f32, b.1 as f32, b.2 as f32);
        w.draw_line(&p_a, &p_b, &Point3::new(1.0, 1.0, 1.0));
    }
}

pub fn axis(w: &mut Window, bounds: &RangeInclusive<f64>) {

    w.draw_line(&Point3::new(*bounds.start() as f32, 0.0, 0.0), &Point3::new(*bounds.end() as f32, 0.0, 0.0), &Point3::new(1.0, 0.0, 0.0));
    w.draw_line(&Point3::new(0.0, *bounds.start() as f32, 0.0), &Point3::new(0.0, *bounds.end() as f32, 0.0), &Point3::new(0.0, 1.0, 0.0));
    w.draw_line(&Point3::new(0.0, 0.0, *bounds.start() as f32), &Point3::new(0.0, 0.0, *bounds.end() as f32), &Point3::new(0.0, 0.0, 1.0));
} 

pub fn render() {
    
    // let mut window = Window::new("Kiss3d: wavelet");

    // let f = std::fs::read_to_string("test.txt").unwrap();
    // let p = parse::str_parse(&f);
    // let mut ctx = convert::convert(p).simplify(false);
    
    
    // //let _ = std::fs::write("output", format!("{:#00.2?}", o));
    
    // window.set_light(Light::StickToCamera);
    
    // let mut t: u64 = 0;
    // let mut last_timestamp = Instant::now();
    
    // while window.render() {
    //     let delta_time = last_timestamp.elapsed();
        
    //     last_timestamp = Instant::now();
    //     t += 1;
    //     let b_start = -3.0 + (0.01 * t as f64).sin() * 1.0;
    //     let b_end = 3.0 + (0.01 * t as f64).sin() * 1.0;
    //     let bounds = b_start..=b_end;
        
    //     if t % 10 == 0 {
    //         let f = std::fs::read_to_string("test.txt").unwrap();
    //         let p = parse::str_parse(&f);
    //         ctx = convert::convert(p).simplify(false);
    //     }
        
    //     let o = ctx
    //     .evaluate_with_x("out", bounds.clone(), 500, false).unwrap()
    //     .into_iter().map(|v| expr::f_2_c((v.0, v.1.unwrap()))).collect::<Vec<(f64, f64, f64)>>();

    //     let calc_time = last_timestamp.elapsed();
    //     axis(&mut window, &bounds);
    //     line(&mut window, &o);

        
    //     println!("frame rendered in {:00.2?} (~{:00.2?} fps) - calculation time: {:00.2?}", delta_time, 1.0 / delta_time.as_secs_f64(), calc_time);
    // }
}