
/*
This is where the parsel trees are converted into a `Context`
*/

use num::ToPrimitive;
use parsel::ast::LeftAssoc;

// * I would expand these, but there are duplicate names in these modules,
// * so I just refer to their whole name instead.
use crate::parse;
use crate::expr::{self, f::*};

pub fn convert(statements: Vec<parse::Statement>) -> expr::Context {
    let mut c = expr::Context::new();

    for statement in statements {
        match statement {
            parse::Statement::Let { kw_let: _, name, kw_eq: _, body, kw_semi: _ } => {
                c.def_var(&name.to_string(), *convert_expr(body));
            }
            parse::Statement::Fn { kw_fn: _, recursive, name, args, kw_eq: _, body , kw_semi: _} => {
                c.def_func(&name.to_string(), recursive.is_some(), args.into_inner().iter().map(|n| n.to_string()).collect(), *convert_expr(body));
            }
        }
    }
    return c;
}

fn convert_expr(e: parse::Expr) -> Box<expr::Expr> {
    match e {
        LeftAssoc::Binary { lhs, op, rhs } => {
            let a = convert_expr(*lhs);
            let b = convert_expr(parsel::ast::LeftAssoc::Rhs(rhs));
            match op {
                parse::AddOp::Add(_) => add(a, b),
                parse::AddOp::Sub(_) => sub(a, b),
            }
        },
        LeftAssoc::Rhs(e) => {
            match e {
                LeftAssoc::Binary { lhs, op, rhs } => {
                    let a = convert_expr(parsel::ast::LeftAssoc::Rhs(*lhs));
                    let b = convert_expr(parsel::ast::LeftAssoc::Rhs(parsel::ast::LeftAssoc::Rhs(rhs)));
                    match op {
                        parse::MulOp::Mul(_) => mul(a, b),
                        parse::MulOp::Div(_) => div(a, b),
                    }
                },
                LeftAssoc::Rhs(e) => {
                    match e {
                        LeftAssoc::Binary { lhs, op, rhs } => {
                            let a = convert_expr(parsel::ast::LeftAssoc::Rhs(parsel::ast::LeftAssoc::Rhs(*lhs)));
                            let b = convert_expr(parsel::ast::LeftAssoc::Rhs(parsel::ast::LeftAssoc::Rhs(parsel::ast::LeftAssoc::Rhs(rhs))));
                            match op {
                                parse::PowOp::Pow(_) => pow(a, b),
                            }
                        },
                        LeftAssoc::Rhs(n) => convert_node(n),
                    }
                    
                },
            }
        },
    }
}

fn convert_node(n: parse::Node) -> Box<expr::Expr> {
    match n {
        parse::Node::Ln(_, a) => panic!()/*ln(convert_expr(*a.into_inner()))*/,
        parse::Node::Log(_, a, b) => panic!()/*log(convert_expr(*a.into_inner()), convert_expr(*b.into_inner()))*/,
        parse::Node::Root(_, a, b) => panic!()/*root(convert_expr(*a.into_inner()), convert_expr(*b.into_inner()))*/,
        parse::Node::Sqrt(_, a) => panic!()/*sqrt(convert_expr(*a.into_inner()))*/,
        parse::Node::Cbrt(_, a) => panic!()/*cbrt(convert_expr(*a.into_inner()))*/,
        parse::Node::Sin(_, a) => panic!()/*sin(convert_expr(*a.into_inner()))*/,
        parse::Node::Cos(_, a) => panic!()/*cos(convert_expr(*a.into_inner()))*/,
        parse::Node::Tan(_, a) => panic!()/*tan(convert_expr(*a.into_inner()))*/,
        parse::Node::Sinh(_, a) => panic!()/*sinh(convert_expr(*a.into_inner()))*/,
        parse::Node::Cosh(_, a) => panic!()/*cosh(convert_expr(*a.into_inner()))*/,
        parse::Node::Tanh(_, a) => panic!()/*tanh(convert_expr(*a.into_inner()))*/,
        parse::Node::Asin(_, a) => panic!()/*asin(convert_expr(*a.into_inner()))*/,
        parse::Node::Acos(_, a) => panic!()/*acos(convert_expr(*a.into_inner()))*/,
        parse::Node::Atan(_, a) => panic!()/*atan(convert_expr(*a.into_inner()))*/,
        parse::Node::Asinh(_, a) => panic!()/*asinh(convert_expr(*a.into_inner()))*/,
        parse::Node::Acosh(_, a) => panic!()/*acosh(convert_expr(*a.into_inner()))*/,
        parse::Node::Atanh(_, a) => panic!()/*atanh(convert_expr(*a.into_inner()))*/,
        parse::Node::Neg(_, a) => neg(convert_expr(*a)),
        parse::Node::Abs(_, a, _) => panic!()/*abs(convert_expr(*a))*/,
        parse::Node::Fn(name, args) => func(name.to_string(), args.into_inner().into_iter().map(|a| convert_expr(a)).collect()),
        parse::Node::Paren(a) => convert_expr(*a.into_inner()),
        parse::Node::Term(t) => Box::new(expr::Expr::Term(convert_term(t))),
    }
}

fn convert_term(t: parse::Term) -> expr::Term {
    match t {
        parse::Term::Var(ident) => expr::Term::Var(ident.to_string()),
        parse::Term::Float(lit_float) => expr::Term::Real(lit_float.into_inner().to_f64().unwrap()),
        parse::Term::Int(lit_int) => expr::Term::Real(lit_int.into_inner().to_f64().unwrap()),
    }
}