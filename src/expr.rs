
use compact_str::CompactString;
use num::complex::Complex64;
use num::traits::{Pow, Inv};
use num::{Zero, One};
use num_complex::ComplexFloat;
use std::collections::HashMap;
use std::ops::{Add, Mul, Neg, RangeInclusive};
use std::time::Instant;

use crate::{C_DEBUG_LEVEL, E_DEBUG_LEVEL};

#[allow(dead_code)]
pub mod f {

    use super::{Expr, Term};

    macro_rules! trig_fn {
        ($f:ident, $n:ident) => {
            pub fn $f(a: Box<Expr>) -> Box<Expr> { Box::new(Expr::$n(a)) }
        };
    }

    pub fn num(n: f64) -> Box<Expr> {
        Box::new(Expr::Term(n.into()))
    }
    pub fn var(s: &str) -> Box<Expr> {
        Box::new(Expr::Term(Term::Var(s.into())))
    }

    pub fn add(a: Box<Expr>, b: Box<Expr>) -> Box<Expr>  { Box::new(Expr::Add(a, b)) }
    pub fn sub(a: Box<Expr>, b: Box<Expr>) -> Box<Expr>  { Box::new(Expr::Add(a, neg(b))) }
    pub fn mul(a: Box<Expr>, b: Box<Expr>) -> Box<Expr>  { Box::new(Expr::Mul(a, b)) }
    pub fn div(a: Box<Expr>, b: Box<Expr>) -> Box<Expr>  { Box::new(Expr::Mul(a, inv(b))) }
    pub fn pow(a: Box<Expr>, b: Box<Expr>) -> Box<Expr>  { Box::new(Expr::Pow(a, b)) }

    /// Root a of b
    pub fn root(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> { Box::new(Expr::Pow(b, Box::new(Expr::Inv(a)))) }

    pub fn ln(a: Box<Expr>) -> Box<Expr> { Box::new(Expr::Ln(a)) }

    /// Log base a of b
    pub fn log(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> { div(ln(b), ln(a)) }

    pub fn abs(a: Box<Expr>) -> Box<Expr> { Box::new(Expr::Abs(a)) }
    pub fn neg(a: Box<Expr>) -> Box<Expr> { Box::new(Expr::Neg(a)) }
    pub fn inv(a: Box<Expr>) -> Box<Expr> { Box::new(Expr::Inv(a)) }

    pub fn sq(a: Box<Expr>) -> Box<Expr> { Box::new(Expr::Pow(a, num(2.0))) }
    pub fn cb(a: Box<Expr>) -> Box<Expr> { Box::new(Expr::Pow(a, num(3.0))) }

    pub fn sqrt(a: Box<Expr>) -> Box<Expr> { root(a, num(2.0)) }
    pub fn cbrt(a: Box<Expr>) -> Box<Expr> { root(a, num(3.0)) }

    pub fn func(n: &str, a: Vec<Box<Expr>>) -> Box<Expr> { Box::new(Expr::Fn(n.to_string(), a.iter().map(|v| *v.clone()).collect())) }

    trig_fn!(sin, Sin);
    trig_fn!(cos, Cos);
    trig_fn!(tan, Tan);
    trig_fn!(sinh, Sinh);
    trig_fn!(cosh, Cosh);
    trig_fn!(tanh, Tanh);
    trig_fn!(asin, Asin);
    trig_fn!(acos, Acos);
    trig_fn!(atan, Atan);
    trig_fn!(asinh, Asinh);
    trig_fn!(acosh, Acosh);
    trig_fn!(atanh, Atanh);

}

pub fn fmt_1(n: (f64, Term)) -> CompactString {
    match n.1 {
        Term::Float(a) => format!("({}, {})", n.0, a).into(),
        Term::Complex(a) => format!("{:00.2?}, {:00.2?}, {:00.2?}", n.0, a.re, a.im).into(),
        Term::Var(_) => panic!(),
    }
}

pub fn f_2_c(n: (f64, Term)) -> (f64, f64, f64) {
    match n.1 {
        Term::Complex(a) => (n.0, a.re, a.im),
        _ => panic!(),
    }
}

// pub fn fmt_2(n: (f64, f64, Term)) -> String {
//     match n.2 {
//         Term::Float(a) => format!("({}, {}, {})", n.0, n.1, a),
//         Term::Complex(a) => format!("({}, {}, {}, {})", n.0, n.1, a.re, a.im),
//         Term::Var(_) => panic!(),
//     }
// }


#[derive(Debug, Clone)]
pub struct Context {
    vars: HashMap<String, Expr>,
    funcs: HashMap<String, Func>,
}
impl Context {
    /// Creates a new `Context`
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        }
        .var("pi", std::f64::consts::PI.into())
        .var("e", std::f64::consts::E.into())
        .var("i", Term::Complex(Complex64::i()).into())
    }

    /// Add a variable to the `Context`
    pub fn var(mut self, name: &str, val: Expr) -> Self {
        self.vars.insert(name.to_string(), val);
        return self;
    }

    /// Add a function to the `Context`
    pub fn func(mut self, name: &str, args: Vec<&str>, body: Expr) -> Self {
        self.funcs.insert(name.to_string(), Func {args: args.iter().map(|n| n.to_string()).collect(), body});
        return self;
    }

    /// Add a function to the `Context`. This version takes a vec of strings for ease of use in convert.rs.
    pub fn func_string(mut self, name: &str, args: Vec<String>, body: Expr) -> Self {
        self.funcs.insert(name.to_string(), Func {args, body});
        return self;
    }

    /// Simplify all the expressions in the current context as much as possible.
    /// 
    /// Functions are expanded to be in line.
    /// 
    /// Has the following steps (in order):
    /// - Simplifies constant expressions into constant values as far as possible
    /// (returns directly from a constant reduction if the value(s) were not further simplified)
    /// - Simplifies some special cases: a+0, 0+b, a\*0, 0\*b, a\*1, 1\*b, a^0, 0^b, a^1, 1^b 
    /// (returns directly from these cases)
    /// - Rearranges terms in nested expressions with the same order of operations to 
    /// maximize constant reduction and minimize the depth of the expression tree
    pub fn simplify(mut self, print: bool) -> Self {
        let timer = Instant::now();
        // Attempt to simplify variables
        for k in self.vars.clone().keys() {
            if C_DEBUG_LEVEL >= 1 {println!("\nsimplify {k:?}")};
            let e = self.vars.get(k).unwrap();
            let n = e.clone().try_simplify(&self);
            self.vars.insert(k.clone(), n);
        };
        if print {println!("Simplified in {:?}", timer.elapsed())};
        return self.trim_funcs();
    }

    fn trim_funcs(mut self) -> Self {
        self.funcs = HashMap::new();
        return self;
    }

    /// Attempts to completely evaluate a variable
    pub fn evaluate(&self, var: &str) -> Option<Term> {
        let v = self.vars.get(var).unwrap_or_else(|| panic!("var `{var}` not found!")).clone();
        if E_DEBUG_LEVEL >= 1 {println!("evaluate {var} : {v:?}")};
        v.try_eval(&self)
    }

    // /// Attempts to completely evaluate a variable with given inputs
    // pub fn evaluate_with_inputs(&self, var: &str, i: Vec<(&str, Term)>) -> Option<Term> {
    //     let mut c = self.clone();
    //     for (n,v) in i.iter() {
    //         c.vars.insert(n.to_string(), v.clone().into());
    //     };
    //     c.evaluate(var)
    // }

    /// Attempts to evaluate a variable for each x in a range
    pub fn evaluate_with_x(&self, var: &str, range: RangeInclusive<f64>, steps: usize, print: bool) -> Option<Vec<(f64, Option<Term>)>> {
        let timer = Instant::now();
        let mut c = self.clone();
        let step = (range.end() - range.start()) / (steps as f64);
        let mut out = Vec::new();
        for s in 0..=steps {
            let x = (s as f64 * step) + range.start();
            c.vars.insert("x".to_string(), x.into());
            // println!();
            out.push((x, c.evaluate(var)));
        };
        if print {println!("calculated {steps} points in {:?}", timer.elapsed())};
        return Some(out);
    }

    #[allow(unused_variables)]
    pub fn evaluate1_with(&self, var: &str, i: &str, range: RangeInclusive<f64>, steps: usize, print: bool) -> (f64, Option<Term>) {
        unimplemented!()
    }

    #[allow(dead_code)]
    pub fn as_fn(&self, _: &str) -> Box<dyn Fn(f64) -> Option<Term>> {
        unimplemented!()
    }

    pub fn evaluate_with_xy(&self, var: &str, x_range: RangeInclusive<f64>, x_steps: usize, y_range: RangeInclusive<f64>, y_steps: usize) -> Option<Vec<(f64, f64, Option<Term>)>> {
        let timer = Instant::now();
        let mut c = self.clone();
        let x_step = (x_range.end() - x_range.start()) / (x_steps as f64);
        let y_step = (y_range.end() - y_range.start()) / (y_steps as f64);
        let mut out = Vec::new();
        for xs in 0..=x_steps {
            let x = (xs as f64 * x_step) + x_range.start();
            c.vars.insert("x".to_string(), x.into());
            c = c.simplify(false);
            // println!();
            for ys in 0..=y_steps {
                let y = (ys as f64 * y_step) + y_range.start();
                c.vars.insert("y".to_string(), y.into());
                out.push((x, y, c.evaluate(var)));
            }
        };
        println!("calculated {} points in {:?}", x_steps*y_steps, timer.elapsed());
        return Some(out);
    }

    fn in_fn(&self, name: String, args: Vec<Expr>) -> Self {
        let mut s = self.clone();
        let f = s.funcs.get(&name).unwrap();
        for i in 0..args.len() {
            if !s.vars.contains_key(&f.args[i]) {
                s.vars.insert(f.args[i].clone(), args[i].clone());
            }
        };
        return s;
    }

    fn try_fn_eval(&self, name: String, args: Vec<Expr>) -> Option<Term> {
        let f = self.funcs.get(&name)?;
        f.body.clone().try_eval(&self.in_fn(name, args))
    }

}

#[derive(Debug, Clone)]
pub struct Func {
    args: Vec<String>,
    body: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Term(Term),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Ln(Box<Expr>),
    Neg(Box<Expr>),
    Inv(Box<Expr>),
    Abs(Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Tan(Box<Expr>),
    Sinh(Box<Expr>),
    Cosh(Box<Expr>),
    Tanh(Box<Expr>),
    Asin(Box<Expr>),
    Acos(Box<Expr>),
    Atan(Box<Expr>),
    Asinh(Box<Expr>),
    Acosh(Box<Expr>),
    Atanh(Box<Expr>),
    Fn(String, Vec<Expr>),
}
impl Expr {
    /// Simplifies the current `Expr` as much as possible.
    fn try_simplify(self, c: &Context) -> Expr {
        if C_DEBUG_LEVEL >= 2 {println!(" - try_simplify {self:?}")};
        /*
        rearrangement example (e is considered a constant in this example, as it will always be a defined variable in any instance of `Context` made with `Contect::new()`):

        o: mul
            -oa: 17.0
            -ob: mul
                -obta: e
                -obtb: t
        
        if oa is const:
            if ob is `mul`:
                if obta is const:
                    -mul
                        -obtb: t
                        -mul
                            -oa: 17.0
                            -obta: e
                    then simplify
                check if obtb is const and rearrange
        check if ob is const:
            check if oa is `mul`:
                check if oata is const and rearrange
                check if oatb is const and rearrange

        if conditions are right, it swaps the nodes around so the two constant terms are grouped together,
        so it can then be simplified into a `Term` and therefore the depth of the tree is reduced.
        */
        match &self {
            Expr::Term(term) => term.expand(c).unwrap_or(self),
            Expr::Add(a, b) => {
                if let (Some(ta), Some(tb)) = (a.try_term(), b.try_term()) {
                    if !ta.is_var() & !tb.is_var() {
                        return Expr::Term(ta.add(tb));
                    };
                }
                let oa = a.clone().try_simplify(c);
                let ob = b.clone().try_simplify(c);
                let o = Expr::Add(Box::new(oa.clone()), Box::new(ob.clone()));
                if !o.has_var() {return o.try_simplify(c)}
                // Special cases (a+0, 0+b)
                if let Expr::Term(ta) = oa.clone() {
                    if ta.is_zero() { // 0 + b
                        return ob;
                    }
                }
                if let Expr::Term(tb) = ob.clone() {
                    if tb.is_zero() { // a + 0
                        return oa;
                    }
                }
                // Rearrangement
                if let Expr::Term(ta) = oa.clone() { // is oa const
                    if let Expr::Add(oba, obb) = ob.clone() { // is ob `add`
                        if let Expr::Term(obta) = *oba { // is obta const
                            return Expr::Add(
                                obb,
                                Box::new(Expr::Add(
                                    Box::new(ta.into()),
                                    Box::new(obta.into()),
                                ))
                            ).try_simplify(c);
                        }
                        if let Expr::Term(obtb) = *obb { // is obtb const
                            return Expr::Add(
                                oba,
                                Box::new(Expr::Add(
                                    Box::new(ta.into()),
                                    Box::new(obtb.into()),
                                ))
                            ).try_simplify(c);
                        }
                    }
                }
                if let Expr::Term(ta) = ob.clone() { // is ob const
                    if let Expr::Add(oaa, oab) = oa.clone() { // is oa `add`
                        if let Expr::Term(oata) = *oaa { // is oata const
                            return Expr::Add(
                                oab,
                                Box::new(Expr::Add(
                                    Box::new(ta.into()),
                                    Box::new(oata.into()),
                                ))
                            ).try_simplify(c);
                        }
                        if let Expr::Term(oatb) = *oab { // is obtb const
                            return Expr::Add(
                                oaa,
                                Box::new(Expr::Add(
                                    Box::new(ta.into()),
                                    Box::new(oatb.into()),
                                ))
                            ).try_simplify(c);
                        }
                    }
                }
                return o;
            },
            Expr::Mul(a, b) => {
                if let (Some(ta), Some(tb)) = (a.try_term(), b.try_term()) {
                    if !ta.is_var() & !tb.is_var() {
                        return Expr::Term(ta.mul(tb));
                    };
                }
                let oa = a.clone().try_simplify(c);
                let ob = b.clone().try_simplify(c);
                let o = Expr::Mul(Box::new(oa.clone()), Box::new(ob.clone()));
                if !o.has_var() {return o.try_simplify(c)}
                // Special cases (a*1, 1*b)
                if let Expr::Term(ta) = oa.clone() {
                    if ta.is_one() { // 1 * b
                        return ob;
                    }
                }
                if let Expr::Term(tb) = ob.clone() {
                    if tb.is_one() { // a * 1
                        return oa;
                    }
                }
                // Special cases (a*0, 0*b)
                if let Expr::Term(ta) = oa.clone() {
                    if ta.is_zero() { // 0 * b
                        return Term::Float(f64::zero()).into();
                    }
                }
                if let Expr::Term(tb) = ob.clone() {
                    if tb.is_zero() { // a * 0
                        return Term::Float(f64::zero()).into();
                    }
                }
                // Rearrangement
                if let Expr::Term(ta) = oa.clone() { // is oa const
                    if let Expr::Mul(oba, obb) = ob.clone() { // is ob `Mul`
                        if let Expr::Term(obta) = *oba { // is obta const
                            return Expr::Mul(
                                obb,
                                Box::new(Expr::Mul(
                                    Box::new(ta.into()),
                                    Box::new(obta.into()),
                                ))
                            ).try_simplify(c);
                        }
                        if let Expr::Term(obtb) = *obb { // is obtb const
                            return Expr::Mul(
                                oba,
                                Box::new(Expr::Mul(
                                    Box::new(ta.into()),
                                    Box::new(obtb.into()),
                                ))
                            ).try_simplify(c);
                        }
                    }
                }
                if let Expr::Term(ta) = ob.clone() { // is ob const
                    if let Expr::Mul(oaa, oab) = oa.clone() { // is oa `Mul`
                        if let Expr::Term(oata) = *oaa { // is oata const
                            return Expr::Mul(
                                oab,
                                Box::new(Expr::Mul(
                                    Box::new(ta.into()),
                                    Box::new(oata.into()),
                                ))
                            ).try_simplify(c);
                        }
                        if let Expr::Term(oatb) = *oab { // is obtb const
                            return Expr::Mul(
                                oaa,
                                Box::new(Expr::Mul(
                                    Box::new(ta.into()),
                                    Box::new(oatb.into()),
                                ))
                            ).try_simplify(c);
                        }
                    }
                }
                return o;
            },
            Expr::Pow(a, b) => {
                if let (Some(ta), Some(tb)) = (a.try_term(), b.try_term()) {
                    if !ta.is_var() & !tb.is_var() { 
                        return Expr::Term(ta.pow(tb));
                    };
                }
                let oa = a.clone().try_simplify(c);
                let ob = b.clone().try_simplify(c);
                let o = Expr::Pow(Box::new(oa.clone()), Box::new(ob.clone()));
                if !o.has_var() {return o.try_simplify(c)}
                // Special cases (a^1, 1^b)
                if let Expr::Term(ta) = oa.clone() {
                    if ta.is_one() { // 1 ^ b = 1
                        return Term::Float(f64::one()).into();
                    }
                }
                if let Expr::Term(tb) = ob.clone() {
                    if tb.is_one() { // a ^ 1 = a
                        return oa;
                    }
                }
                // Special cases (a^0, 0^b)
                if let Expr::Term(ta) = oa.clone() {
                    if ta.is_zero() { // 0 ^ b = 0
                        return Term::Float(f64::zero()).into();
                    }
                }
                if let Expr::Term(tb) = ob.clone() {
                    if tb.is_zero() { // a ^ 0 = 1
                        return Term::Float(f64::one()).into();
                    }
                }
                if let Expr::Term(ta) = oa.clone() { // is oa const
                    if let Expr::Pow(oba, obb) = ob.clone() { // is ob `Pow`
                        if let Expr::Term(obta) = *oba { // is obta const
                            return Expr::Pow(
                                obb,
                                Box::new(Expr::Pow(
                                    Box::new(ta.into()),
                                    Box::new(obta.into()),
                                ))
                            ).try_simplify(c);
                        }
                        if let Expr::Term(obtb) = *obb { // is obtb const
                            return Expr::Pow(
                                oba,
                                Box::new(Expr::Pow(
                                    Box::new(ta.into()),
                                    Box::new(obtb.into()),
                                ))
                            ).try_simplify(c);
                        }
                    }
                }
                if let Expr::Term(ta) = ob.clone() { // is ob const
                    if let Expr::Pow(oaa, oab) = oa.clone() { // is oa `Pow`
                        if let Expr::Term(oata) = *oaa { // is oata const
                            return Expr::Pow(
                                oab,
                                Box::new(Expr::Pow(
                                    Box::new(ta.into()),
                                    Box::new(oata.into()),
                                ))
                            ).try_simplify(c);
                        }
                        if let Expr::Term(oatb) = *oab { // is obtb const
                            return Expr::Pow(
                                oaa,
                                Box::new(Expr::Pow(
                                    Box::new(ta.into()),
                                    Box::new(oatb.into()),
                                ))
                            ).try_simplify(c);
                        }
                    }
                }
                return o;
            },
            Expr::Ln(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() {
                        return Expr::Term(ta.ln());
                    };
                }
                let o = Expr::Ln(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            }
            Expr::Neg(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.neg()); };
                }
                let o = Expr::Neg(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Inv(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.inv()); };
                }
                let o = Expr::Inv(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Abs(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Abs(a.clone()); };
                    return Expr::Term(ta.abs());
                }
                let o = Expr::Abs(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Sin(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.sin()); };
                }
                let o = Expr::Sin(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Cos(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.cos()); };
                }
                let o = Expr::Cos(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Tan(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.tan()); };
                }
                let o = Expr::Tan(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Sinh(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.sinh()); };
                }
                let o = Expr::Sinh(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Cosh(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.cosh()); };
                }
                let o = Expr::Cosh(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Tanh(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.tanh()); };
                }
                let o = Expr::Tanh(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Asin(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.asin()); };
                }
                let o = Expr::Asin(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Acos(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.acos()); };
                }
                let o = Expr::Acos(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Atan(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.atan()); };
                }
                let o = Expr::Atan(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Asinh(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.asinh()); };
                }
                let o = Expr::Asinh(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Acosh(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.acosh()); };
                }
                let o = Expr::Acosh(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Atanh(a) => {
                if let Some(ta) = a.try_term() {
                    if !ta.is_var() { return Expr::Term(ta.atanh()); };
                }
                let o = Expr::Atanh(Box::new(a.clone().try_simplify(c)));
                if !o.has_var() {return o.try_simplify(c)}
                return o;
            },
            Expr::Fn(n, args) => {
                let func = c.funcs.get(n).unwrap();
                let mut body = func.body.clone();
                for i in 0..func.args.len() {
                    Self::replace_var(&mut body, func.args[i].clone(), args[i].clone(), c);
                };
                return body.try_simplify(c);
            },
        }
    }

    fn has_var(&self) -> bool {
        match self {
            Expr::Term(a) => a.is_var(),
            Expr::Add(a, b) => a.has_var() | b.has_var(),
            Expr::Mul(a, b) => a.has_var() | b.has_var(),
            Expr::Pow(a, b) => a.has_var() | b.has_var(),
            Expr::Ln(a) => a.has_var(),
            Expr::Neg(a) => a.has_var(),
            Expr::Inv(a) => a.has_var(),
            Expr::Abs(a) => a.has_var(),
            Expr::Sin(a) => a.has_var(),
            Expr::Cos(a) => a.has_var(),
            Expr::Tan(a) => a.has_var(),
            Expr::Sinh(a) => a.has_var(),
            Expr::Cosh(a) => a.has_var(),
            Expr::Tanh(a) => a.has_var(),
            Expr::Asin(a) => a.has_var(),
            Expr::Acos(a) => a.has_var(),
            Expr::Atan(a) => a.has_var(),
            Expr::Asinh(a) => a.has_var(),
            Expr::Acosh(a) => a.has_var(),
            Expr::Atanh(a) => a.has_var(),
            Expr::Fn(_, args) => args.iter().map(|n| n.has_var()).collect::<Vec<bool>>().contains(&true),
        }
    }

    /// Goes through and recursively replaces all the instances of a variable with some expression.
    fn replace_var(s: &mut Self, var: String, val: Expr, c: &Context) {
        if C_DEBUG_LEVEL >= 3 {println!("   - replace_var {var:?} with {val:?} in {s:?}")};
        match s {
            Expr::Term(Term::Var(n)) => if n.clone() == var {*s = val;},
            Expr::Term(_) => return,
            Expr::Add(a, b) => {Self::replace_var(a, var.clone(), val.clone(), c); Self::replace_var(b, var.clone(), val.clone(), c);},
            Expr::Mul(a, b) => {Self::replace_var(a, var.clone(), val.clone(), c); Self::replace_var(b, var.clone(), val.clone(), c);},
            Expr::Pow(a, b) => {Self::replace_var(a, var.clone(), val.clone(), c); Self::replace_var(b, var.clone(), val.clone(), c);},
            Expr::Ln(a) => Self::replace_var(a, var, val, c),
            Expr::Neg(a) => Self::replace_var(a, var, val, c),
            Expr::Inv(a) => Self::replace_var(a, var, val, c),
            Expr::Abs(a) => Self::replace_var(a, var, val, c),
            Expr::Sin(a) => Self::replace_var(a, var, val, c),
            Expr::Cos(a) => Self::replace_var(a, var, val, c),
            Expr::Tan(a) => Self::replace_var(a, var, val, c),
            Expr::Sinh(a) => Self::replace_var(a, var, val, c),
            Expr::Cosh(a) => Self::replace_var(a, var, val, c),
            Expr::Tanh(a) => Self::replace_var(a, var, val, c),
            Expr::Asin(a) => Self::replace_var(a, var, val, c),
            Expr::Acos(a) => Self::replace_var(a, var, val, c),
            Expr::Atan(a) => Self::replace_var(a, var, val, c),
            Expr::Asinh(a) => Self::replace_var(a, var, val, c),
            Expr::Acosh(a) => Self::replace_var(a, var, val, c),
            Expr::Atanh(a) => Self::replace_var(a, var, val, c),
            Expr::Fn(_, args) => {
                for i in 0..args.len() {
                    Self::replace_var(&mut args[i], var.clone(), val.clone(), c);
                }
            },

        }

        if C_DEBUG_LEVEL >= 3 { println!("   {s:?}"); };
    }

    fn try_eval(self, c: &Context) -> Option<Term> {
        // println!("{self:?} <- {:?}", c.vars);
        if E_DEBUG_LEVEL >= 2 {println!(" - {self:?}")};
        if E_DEBUG_LEVEL >= 3 {println!("   - {:?}", c.vars)};
        Some(match self {
            Expr::Term(term) => term.unvar(c)?,
            Expr::Add(a, b) => {a.try_eval(c)?.add(b.try_eval(c)?)},
            Expr::Mul(a, b) => {a.try_eval(c)?.mul(b.try_eval(c)?)},
            Expr::Pow(a, b) => {a.try_eval(c)?.pow(b.try_eval(c)?)},
            Expr::Ln(a) => {a.try_eval(c)?.ln()},
            Expr::Neg(a) => {a.try_eval(c)?.neg()},
            Expr::Inv(a) => {a.try_eval(c)?.inv()},
            Expr::Abs(a) => {a.try_eval(c)?.abs()},
            Expr::Sin(a) => {a.try_eval(c)?.sin()},
            Expr::Cos(a) => {a.try_eval(c)?.cos()},
            Expr::Tan(a) => {a.try_eval(c)?.tan()},
            Expr::Sinh(a) => {a.try_eval(c)?.sinh()},
            Expr::Cosh(a) => {a.try_eval(c)?.cosh()},
            Expr::Tanh(a) => {a.try_eval(c)?.tanh()},
            Expr::Asin(a) => {a.try_eval(c)?.asin()},
            Expr::Acos(a) => {a.try_eval(c)?.acos()},
            Expr::Atan(a) => {a.try_eval(c)?.atan()},
            Expr::Asinh(a) => {a.try_eval(c)?.asinh()},
            Expr::Acosh(a) => {a.try_eval(c)?.acosh()},
            Expr::Atanh(a) => {a.try_eval(c)?.atanh()},
            Expr::Fn(name, args) => {c.try_fn_eval(name, args)?},
        })
    }
    
    fn try_term(&self) -> Option<Term> {
        match self {
            Self::Term(t) => Some(t.clone()),
            _ => None,
        }
    }
}
impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Expr::Term(value.into())
    }
}
impl From<Term> for Expr {
    fn from(value: Term) -> Self {
        Expr::Term(value)
    }
}


macro_rules! impl_trig {
    ($f:ident) => {
        fn $f(self) -> Self {
            match self {
                Term::Var(_) => panic!(),
                Term::Float(n) => Term::Float(n.$f()),
                Term::Complex(n) => Term::Complex(n.$f()),
            }
        }
    };
}



#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(String),
    Float(f64),
    Complex(Complex64),
}
impl Term {

    impl_trig!(sin);
    impl_trig!(cos);
    impl_trig!(tan);
    impl_trig!(sinh);
    impl_trig!(cosh);
    impl_trig!(tanh);
    impl_trig!(asin);
    impl_trig!(acos);
    impl_trig!(atan);
    impl_trig!(asinh);
    impl_trig!(acosh);
    impl_trig!(atanh);

    fn is_var(&self) -> bool {
        match &self {
            Self::Var(_) => true,
            _ => false,
        }
    }

    fn abs(self) -> Term {
        match self {
            Term::Var(_) => panic!(),
            Term::Float(n) => Term::Float(n.abs()),
            Term::Complex(n) => Term::Float(n.norm()),
        }
    }

    fn pow(self, rhs: Self) -> Term {
        match (self, rhs) {
            (Term::Float(a), Term::Float(b)) => a.pow(b).into(),
            (Term::Complex(a), Term::Float(b)) => Term::Complex(a.powf(b)),
            (Term::Float(a), Term::Complex(b)) => Term::Complex(a.powc(b)),
            (Term::Complex(a), Term::Complex(b)) => Term::Complex(a.powc(b)),
            _ => panic!(),
        }
    }

    fn ln(self) -> Term {
        match self {
            Term::Var(_) => panic!(),
            Term::Float(a) => a.ln().into(),
            Term::Complex(a) => Term::Complex(a.ln()),
        }
    }

    fn unvar(&self, c: &Context) -> Option<Self> {
        Some(match self {
            Term::Var(n) => {
                // println!("unvar {n}!");
                c.evaluate(&n[..])?
            },
            Term::Float(_) => self.clone(),
            Term::Complex(_) => self.clone(),
        })
    }

    /// Tries to expand the variable. If `self` is not a variable, returns `None`.
    fn expand(&self, c: &Context) -> Option<Expr> {
        Some(match self {
            Term::Var(n) => {
                c.vars.get(n)?.clone()
            },
            _ => return None,
        })
    }

    /// Checks if the value of `self` is 1 of some kind.
    fn is_one(&self) -> bool {
        match self {
            Term::Float(1.0) => true,
            Term::Complex(Complex64::ONE) => true,
            _ => false,
        }
    }

    /// Checks if the value of `self` is 0 of some kind.
    fn is_zero(&self) -> bool {
        match self {
            Term::Float(0.0) => true,
            Term::Complex(Complex64::ZERO) => true,
            _ => false,
        }
    }
}
impl Add for Term {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Term::Var(_), _) => panic!(),
            (_, Term::Var(_)) => panic!(),
            (Term::Float(a), Term::Float(b)) => Term::Float(a + b),
            (Term::Float(a), Term::Complex(b)) => Term::Complex(a + b),
            (Term::Complex(a), Term::Float(b)) => Term::Complex(a + b),
            (Term::Complex(a), Term::Complex(b)) => Term::Complex(a + b),
        }
    }
}
impl Mul for Term {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Term::Var(_), _) => panic!(),
            (_, Term::Var(_)) => panic!(),
            (Term::Float(a), Term::Float(b)) => Term::Float(a * b),
            (Term::Float(a), Term::Complex(b)) => Term::Complex(a * b),
            (Term::Complex(a), Term::Float(b)) => Term::Complex(a * b),
            (Term::Complex(a), Term::Complex(b)) => Term::Complex(a * b),
        }
    }
}
impl Inv for Term {
    type Output = Self;

    fn inv(self) -> Self::Output {
        match self {
            Term::Var(_) => panic!(),
            Term::Float(n) => Term::Float(n.inv()),
            Term::Complex(n) => Term::Complex(n.inv()),
        }
    }
}
impl Neg for Term {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Term::Var(_) => panic!(),
            Term::Float(n) => Term::Float(n.neg()),
            Term::Complex(n) => Term::Complex(n.neg()),
        }
    }
}
impl From<f64> for Term {
    fn from(value: f64) -> Self {
        Term::Float(value)
    }
}
impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n:00.2}"),
            Self::Complex(n) => write!(f, "{n}"),
        }
    }
}