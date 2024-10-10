
/*
This file holds multiple purposesm all surrounding the computation of points
that are heavily intertwined and for that reason, kept in the same file.

Steps taken to calculate points:
- A `Context` is generated.
- `.simplify_for_var()` is called on that context to generate an `Expr` (an
expression tree) and the needed recursive functions. Non recursive functions
are expanded to be inline so they can simplify.
    - The simplification starts by expanding variables to be inline.
    // - Then (non recursive) functions are expanded
    - Constant expressions are reduced as much as possible.
    - Commutable operations are reordered both to group constants together
    and to follow standards that make other steps easier.
    - Constants are reduced again.
    // - Expressions are factored (undistributed) as much as possible. 
    // - Reordering again. // * may be unnessecary
    // - Constant reduction again. // * may be unnessecary
    - Special cases like adding zero are simplified.
    // - Divisions are simplified as much as possible.
    // - Small integer powers are expanded to speed up computation.
*/



use std::{collections::HashMap, ops::{Add, Mul}, str::FromStr};
use num::{pow::Pow, One, Zero};
use num_complex::{Complex64, ComplexFloat};

pub type Exp = Box<Expr>;


pub mod f {
    use super::{Exp, Expr, Term};

    pub fn num(n: f64) -> Exp { Expr::from(Term::from(n)).r#box() }
    pub fn term(t: Term) -> Exp { Expr::from(t).r#box() }

    pub fn neg(a: Exp) -> Exp { mul(num(-1.0), a) }
    pub fn inv(a: Exp) -> Exp { pow(a, num(-1.0)) }

    pub fn add(a: Exp, b: Exp) -> Exp { Expr::Add(a, b).r#box() }
    pub fn mul(a: Exp, b: Exp) -> Exp { Expr::Mul(a, b).r#box() }
    pub fn pow(a: Exp, b: Exp) -> Exp { Expr::Pow(a, b).r#box() }

    pub fn sub(a: Exp, b: Exp) -> Exp { add(a, neg(b)) }
    pub fn div(a: Exp, b: Exp) -> Exp { mul(a, inv(b)) }
}


/// The context of the input. This is essentially just the whole parsed input.
#[derive(Debug, Default)]
pub struct Context {
    vars: HashMap<String, Exp>,
    fns: HashMap<String, Func>,
}
impl Context {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.def_var("e", *f::num(std::f64::consts::E));
        s.def_var("pi", *f::num(std::f64::consts::PI));
        s.def_var("i", *f::term(Complex64::I.into()));
        return s;
    }
    pub fn def_var(&mut self, var: &str, val: Expr) {
        self.vars.insert(var.to_string(), Box::new(val));
    }
    pub fn def_func(&mut self, name: &str, recursive: bool, args: Vec<String>, body: Expr) {
        self.fns.insert(name.to_string(), Func { recursive, args, body });
    }

    /// Checks for illigal recursion. This includes:
    /// - Variables defined using themselves
    /// - Functions not labeled as recursive calling themselves.
    /// Even through misdirection (`f` calls `g` & `g` calls `f`)
    pub fn check_for_illigal_recursion(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();

        // For each variable, check if it is recursive.
        for (var, val) in self.vars.iter() {
            if val.has_var(var, &self) {
                errs.push(format!("ERROR: variable {var} is recursive! variables cannot be recursive."));
            }
        }

        // For each function, check if it's recursive.
        for (name, f) in self.fns.iter() {
            if f.body.has_fn(name, &self) {
                errs.push(format!("ERROR: function {name} is recursive! functions that aren't labeled as so cannot be recursive.\n\t(Try adding `(recursive)` after the `fn` keyword)"));
            }
        }
        
        if errs.is_empty() {return Ok(())} // yay! no errors!
        
        return Err(errs); // ono! errors!
    }

    /// Simplifies a specific variable into an expression and recursive functions.
    pub fn simplify_for_var(&self, var: &str) -> (Expr, HashMap<String, Func>) {
        let mut e = *self.vars.get(&var.to_string()).unwrap().clone();

        e = e.expand_vars(&self.vars.clone().into_iter().collect());
        //e = e.expand_funcs(&self.fns.clone().into_iter().collect());
        e = e.reduce_const();
        e = e.reorder();
        e = e.reduce_const();
        // e = e.factor();
        // e = e.reorder(); // * need to see if this is necessary
        // e = e.reduce_const(); // * need to see if this is necessary
        e = e.special_cases();
        // e = e.simplify_div();
        // e = e.expand_pow();
        
        // Only return the functions that are both recursive and called to evaluate var
        let mut funcs = HashMap::new();
        for (k, v) in self.fns.clone().into_iter().filter(|f| f.1.recursive & e.has_fn(&f.0, &self)).collect::<Vec<(String, Func)>>() {
            funcs.insert(k, v);
        };

        return (e, funcs);
    }
}

#[derive(Debug, Clone)]
pub struct Func {
    recursive: bool,
    args: Vec<String>,
    body: Expr,
}


/// An expression tree node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Term(Term),
    Add(Exp, Exp),
    Mul(Exp, Exp),
    Pow(Exp, Exp),
}
impl Expr {
    /// Expand all instances of a variable into an expression.
    pub fn expand_vars(&self, vars: &Vec<(String, Exp)>) -> Self {
        match self {
            // * the recursive expansion is necessary to be complete with one call of `expand_vars`
            Self::Term(t) => t.expand_vars(vars).expand_vars(vars),
            Self::Add(a, b) => Self::Add((*a).expand_vars(vars).r#box(), (*b).expand_vars(vars).r#box()),
            Self::Mul(a, b) => Self::Mul((*a).expand_vars(vars).r#box(), (*b).expand_vars(vars).r#box()),
            Self::Pow(a, b) => Self::Pow((*a).expand_vars(vars).r#box(), (*b).expand_vars(vars).r#box()),
        }
    }

    /// Expand all instances of a function into an expression.
    pub fn expand_funcs(&self, funcs: &Vec<(String, Func)>) -> Self {unimplemented!()}

    /// Reduce constant nodes into terms.
    pub fn reduce_const(&self) -> Self {
        match self {
            Self::Term(_) => self.clone(),
            Self::Add(a, b) => {
                let a = a.reduce_const();
                let b = b.reduce_const();
                if a.is_const() & b.is_const() {
                    return (a.force_const().add(b.force_const())).into();
                }
                Self::Add(a.r#box(), b.r#box())
            },
            Self::Mul(a, b) => {
                let a = a.reduce_const();
                let b = b.reduce_const();
                if a.is_const() & b.is_const() {
                    return (a.force_const().mul(b.force_const())).into();
                }
                Self::Mul(a.r#box(), b.r#box())
            },
            Self::Pow(a, b) => {
                let a = a.reduce_const();
                let b = b.reduce_const();
                if a.is_const() & b.is_const() {
                    return (a.force_const().pow(b.force_const())).into();
                }
                Self::Pow(a.r#box(), b.r#box())
            },
        }
    }

    /// Reorders some binary operations (add,mul) to put a term first if possible
    /// and follows the conventions of subtraction having the negative last and 
    /// division having the inverse last.
    /// This aids in other simplification processes.
    pub fn reorder(&self) -> Self {
        match self {
            Self::Add(a, b) => {
                match (a.is_term(), b.is_term()) {
                    (true, true) => self.reduce_const(),
                    (false, true) => Self::Add(b.clone().r#box(), a.clone().r#box()), // Switcharoo
                    _ => match (a.is_neg(), b.is_neg()) {
                        (true, false) => Self::Add(b.clone().r#box(), a.clone().r#box()), // Switcharoo
                        _ => self.clone(),
                    }
                }
            },
            Self::Mul(a, b) => {
                match (a.is_term(), b.is_term()) {
                    (true, true) => self.reduce_const(),
                    (false, true) => Self::Mul(b.clone().r#box(), a.clone().r#box()), // Switcharoo
                    _ => match (a.is_inv(), b.is_inv()) {
                        (true, false) => Self::Mul(b.clone().r#box(), a.clone().r#box()), // Switcharoo
                        _ => self.clone(),
                    }
                }
            },
            // Only Add and Mul are commutative.
            _ => self.clone(),
        }
    }

    /// Simplifies special cases like adding 0. Assumes constants have been reduced already.
    pub fn special_cases(&self) -> Self {
        // Special cases for single input operations are unnecessary as all constant inputs for those
        // will get simplified out in constant reduction.
        match self {
            Self::Add(a, b) => {
                if a.is_zero() { return *b.clone() } // 0 + b
                if b.is_zero() { return *a.clone() } // a + 0
                return self.clone();
            },
            Self::Mul(a, b) => {
                if a.is_zero() | b.is_zero() { return *f::num(0.0); } // 0*b | b*0
                if a.is_one() { return *b.clone() } // 1 * b
                if b.is_one() { return *a.clone() } // a * 1
                return self.clone();
            },
            Self::Pow(a, b) => {
                if a.is_zero() { return *f::num(0.0) }  // 0^b
                if b.is_zero() { return *f::num(1.0) }  // a^0
                if a.is_one()  { return *f::num(1.0) }  // 1^b
                if b.is_one()  { return *a.clone() }    // a^1
                return self.clone();
            },
            _ => return self.clone(),
        }
    }

    /// Try to factor out multiplications and additions.
    pub fn factor(&self) -> Self {unimplemented!()}

    /// Try to find common factors in fractions.
    pub fn simplify_div(&self) -> Self {unimplemented!()}

    /// For small integer powers, expand them into multiplication.
    pub fn expand_pow(&self) -> Self {unimplemented!()}

    /// Checks if this expression tree contains a variable.
    /// Search includes other variables in a `Context`.
    pub fn has_var(&self, var: &String, c: &Context) -> bool {
        match self {
            Self::Term(Term::Var(v)) => {
                if v == var {return true}
                if let Some(other_var) = c.vars.get(v) {
                    return other_var.has_var(var, c);
                }
                return false;
            },
            Self::Term(_) => false,
            Self::Add(a, b) => a.has_var(var, c) | b.has_var(var, c),
            Self::Mul(a, b) => a.has_var(var, c) | b.has_var(var, c),
            Self::Pow(a, b) => a.has_var(var, c) | b.has_var(var, c),
        }
    }

    /// Checks if this expression tree contains a function.
    /// Search includes other functions in a `Context`.
    pub fn has_fn(&self, name: &String, c: &Context) -> bool {
        match self {
            Self::Term(_) => false,
            Self::Add(a, b) => a.has_var(name, c) | b.has_var(name, c),
            Self::Mul(a, b) => a.has_var(name, c) | b.has_var(name, c),
            Self::Pow(a, b) => a.has_var(name, c) | b.has_var(name, c),
        }
    }

    /// Checks if `self` is negative
    pub fn is_neg(&self) -> bool {
        match self {
            Self::Mul(a, b) => a.is_neg_one() ^ b.is_neg_one(),
            _ => false,
        }
    }
    /// Checks if `self` is an inverse.
    pub fn is_inv(&self) -> bool {
        match self {
            Self::Pow(_, p) => {
                if p.is_const() {
                    return p.force_const().is_neg_one();
                }
                false
            },
            _ => false,
        }
    }
    /// Checks if `self` is a division.
    pub fn is_div(&self) -> bool {
        match self {
            Self::Mul(a, b) => a.is_inv() ^ b.is_inv(),
            _ => false,
        }
    }
    /// Checks if `self` is a commutative operation.
    pub fn is_commutative(&self) -> bool {
        match self {
            Self::Add(_, _) | Self::Mul(_, _) => true,
            _ => false,
        }
    } 
    /// Checks if `self` is a binary operation (has 2 inputs).
    pub fn is_binary(&self) -> bool {
        match self {
            Self::Add(_, _) | Self::Mul(_, _) | Self::Pow(_, _) => true,
            _ => false,
        }
    }
    /// Checks if `self` is a term.
    pub fn is_term(&self) -> bool {
        match self {
            Self::Term(_) => true,
            _ => false,
        }
    }
    // Checks if `self` is zero.
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Term(t) => t.is_zero(),
            _ => false,
        }
    }
    // Checks if `self` is one.
    pub fn is_one(&self) -> bool {
        match self {
            Self::Term(t) => t.is_one(),
            _ => false,
        }
    }
    /// Checks if `self` is -1 of some kind. Assumes consts have been reduced, does not reduce consts to check.
    pub fn is_neg_one(&self) -> bool {
        match self {
            Self::Term(t) => t.is_neg_one(),
            _ => false,
        }
    }
    /// Checks if `self` is constant.
    pub fn is_const(&self) -> bool {
        match self {
            Self::Term(t) => t.is_const(),
            _ => false,
        }
    }

    /// Force `self` into a const `Term`, panics if it can't.
    pub fn force_const(&self) -> Term {
        match self {
            Self::Term(t) => t.force_const(),
            _ => panic!(),
        }
    }


    /// Boxes up `self`
    pub fn r#box(self) -> Exp {
        Box::new(self)
    }
}
impl From<Term> for Expr {
    fn from(value: Term) -> Self {
        Self::Term(value)
    }
}


/// A term in the expression tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(String),
    Real(f64),
    Complex(Complex64),
}
impl Term {

    /// Checks if `self` is const.
    pub fn is_const(&self) -> bool {
        match self {
            Self::Var(_) => false,
            Self::Real(_) => true,
            Self::Complex(_) => true,
        }
    }

    /// Panics if `self` is not const.
    pub fn force_const(&self) -> Self {
        match self {
            Self::Var(_) => panic!(),
            Self::Real(_) => self.clone(),
            Self::Complex(_) => self.clone(),
        }
    }

    /// Returns an expression if `self` is the Variable `var`.
    pub fn expand_vars(&self, vars: &Vec<(String, Exp)>) -> Expr {
        let mut t = f::term(self.clone());
        for (var, val) in vars {
            match self {
                Self::Var(n) => {if n == var {t = val.clone()}},
                _ => (),
            }
        }
        return *t;
    }

    pub fn is_neg_one(&self) -> bool {
        match self {
            Self::Real(n) => (-n).is_one(),
            Self::Complex(n) => (-n).is_one(),
            _ => false,
        }
    }
}
impl FromStr for Term {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Var(s.to_string()))
    }
}
impl From<String> for Term {
    fn from(value: String) -> Self {
        Term::Var(value)
    }
}
impl From<f64> for Term {
    fn from(value: f64) -> Self {
        Term::Real(value)
    }
}
impl From<Complex64> for Term {
    fn from(value: Complex64) -> Self {
        Term::Complex(value)
    }
}
impl Zero for Term {
    fn zero() -> Self {
        Self::Real(f64::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Real(n) => n.is_zero(),
            Self::Complex(n) => n.is_zero(),
            _ => false,
        }
    }
}
impl One for Term {
    fn one() -> Self {
        Self::Real(f64::one())
    }
    
    fn is_one(&self) -> bool {
        match self {
            Self::Real(n) => n.is_one(),
            Self::Complex(n) => n.is_zero(),
            _ => false,
        }
    }
}
impl Add<Term> for Term {
    type Output = Term;

    fn add(self, rhs: Term) -> Self::Output {
        match (self, rhs) {
            (Term::Real(a), Term::Real(b)) => (a+b).into(),
            (Term::Real(a), Term::Complex(b)) => (a+b).into(),
            (Term::Complex(a), Term::Real(b)) => (a+b).into(),
            (Term::Complex(a), Term::Complex(b)) => (a+b).into(),
            _ => panic!(),
        }
    }
}
impl Mul<Term> for Term {
    type Output = Self;

    fn mul(self, rhs: Term) -> Self::Output {
        match (self, rhs) {
            (Term::Real(a), Term::Real(b)) => (a*b).into(),
            (Term::Real(a), Term::Complex(b)) => (a*b).into(),
            (Term::Complex(a), Term::Real(b)) => (a*b).into(),
            (Term::Complex(a), Term::Complex(b)) => (a*b).into(),
            _ => panic!(),
        }
    }
}
impl Pow<Term> for Term {
    type Output = Self;

    fn pow(self, rhs: Term) -> Self::Output {
        match (self, rhs) {
            (Term::Real(a), Term::Real(b)) => (a.pow(b)).into(),
            (Term::Real(a), Term::Complex(b)) => (a.powc(b)).into(),
            (Term::Complex(a), Term::Real(b)) => (a.pow(b)).into(),
            (Term::Complex(a), Term::Complex(b)) => (a.pow(b)).into(),
            _ => panic!(),
        }
    }
}