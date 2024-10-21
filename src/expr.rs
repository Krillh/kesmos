
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



use std::{collections::HashMap, ops::{Add, Mul}, str::FromStr, usize};
use num::{pow::Pow, One, Zero};
use num_complex::{Complex64, ComplexFloat};

pub type Exp = Box<Expr>;


pub mod f {
    use super::{Exp, Expr, Term};

    pub fn num(n: f64) -> Exp { Expr::from(Term::from(n)).r#box() }
    pub fn term(t: Term) -> Exp { Expr::from(t).r#box() }

    pub fn neg(a: Exp) -> Exp { mul(num(-1.0), a) }
    pub fn inv(a: Exp) -> Exp { pow(a, num(-1.0)) }

    pub fn add(a: Exp, b: Exp) -> Exp { Expr::Add(vec![*a, *b]).r#box() }
    pub fn mul(a: Exp, b: Exp) -> Exp { Expr::Mul(vec![*a, *b]).r#box() }
    pub fn pow(a: Exp, b: Exp) -> Exp { Expr::Pow(a, b).r#box() }

    pub fn sub(a: Exp, b: Exp) -> Exp { add(a, neg(b)) }
    pub fn div(a: Exp, b: Exp) -> Exp { mul(a, inv(b)) }

    pub fn func(name: String, args: Vec<Exp>) -> Exp { Expr::Fn(name, args).r#box() }
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

        println!(" - expanding vars");
        e = e.expand_vars(&self.vars.clone().into_iter().collect());
        println!(" - expanding funcs");
        e = e.expand_funcs(&self.fns.clone().into_iter().collect());
        println!(" - expanding vars");
        e = e.expand_vars(&self.vars.clone().into_iter().collect());
        println!(" - flattening");
        e = e.flatten();
        println!(" - reduce consts");
        e = e.reduce_const();

        // e = e.factor();
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
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    Pow(Exp, Exp),
    Fn(String, Vec<Exp>),
}
impl Expr {
    
    /// Flattens Add and Mul trees
    pub fn flatten(&self) -> Self {
        println!("   - flatten {self:?}");
        match self {
            Self::Term(_) => self.clone(),
            Self::Fn(_, _) => self.clone(),
            Self::Add(n) => Self::Add(n.iter().flat_map(|a| {a.flatten_add()}).collect()),
            Self::Mul(n) => Self::Mul(n.iter().flat_map(|a| {a.flatten_mul()}).collect()),
            Self::Pow(a, b) => Self::Pow(a.flatten().r#box(), b.flatten().r#box()),
        }
    }
    fn flatten_mul(&self) -> Vec<Expr> {
        println!("   - flatten {self:?}");
        match self {
            Self::Mul(n) => n.iter().flat_map(|a| a.flatten_mul()).collect(),
            o => [o.flatten()].to_vec(),
        }
    }
    fn flatten_add(&self) -> Vec<Expr> {
        println!("   - flatten {self:?}");
        match self {
            Self::Add(n) => n.iter().flat_map(|a| a.flatten_add()).collect(),
            o => [o.flatten()].to_vec(),
        }
    }

    /// Expand all instances of a variable into an expression.
    pub fn expand_vars(&self, vars: &Vec<(String, Exp)>) -> Self {
        println!("   - {self:?}");
        match self {
            // * the recursive expansion is necessary to be complete with one call of `expand_vars`
            Self::Term(t) => t.expand_vars(vars),
            Self::Add(n) => Self::Add(n.iter().map(|a| a.expand_vars(vars)).collect()),
            Self::Mul(n) => Self::Mul(n.iter().map(|a| a.expand_vars(vars)).collect()),
            Self::Pow(a, b) => Self::Pow((*a).expand_vars(vars).r#box(), (*b).expand_vars(vars).r#box()),
            Self::Fn(s, n) => Self::Fn(s.clone(), n.iter().map(|a| a.expand_vars(vars).r#box()).collect()),
        }
    }

    /// Expand all instances of a function into an expression.
    pub fn expand_funcs(&self, funcs: &HashMap<String, Func>) -> Self {
        match self {
            Self::Term(_) => self.clone(),
            Self::Fn(name, args) => {
                let f = funcs.get(name).unwrap();
                if f.recursive {
                    return self.clone();
                }
                let b = f.body.expand_vars(&f.args.clone().into_iter().zip(args.iter().map(|a| a.clone().r#box())).collect());
                b.expand_funcs(funcs)
            },
            Self::Add(n) => Self::Add(n.iter().map(|a| a.expand_funcs(funcs)).collect()),
            Self::Mul(n) => Self::Mul(n.iter().map(|a| a.expand_funcs(funcs)).collect()),
            Self::Pow(a,b) => Self::Pow(a.expand_funcs(funcs).r#box(), b.expand_funcs(funcs).r#box()),
        }
    }

    /// Reduce constant nodes into terms.
    pub fn reduce_const(&self) -> Self {
        match self {
            Self::Term(_) => self.clone(),
            Self::Add(n) => {
                // Reduce const for all items
                let mut n: Vec<Expr> = n.iter().map(|a| a.reduce_const()).collect();

                // Sort the items so constants are first, then find the cutoff where the items are no longer 
                // constant.
                n.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                let mut cutoff = usize::MAX;
                for i in 0..n.len() {
                    if !n[i].is_const() {break}
                    cutoff = i+1;
                }

                println!("   - simplify {n:?}");

                // No constants found
                if cutoff == usize::MAX {
                    return Self::Add(n);
                }

                // Accumulate all of the constant values
                let c = n[1..cutoff].iter().fold(n[0].force_const(),|acc, a| acc + a.force_const());

                // If all of the values were constant, return a term.
                if cutoff == n.len() {
                    return *f::term(c);
                }

                // Return the same terms with the reduced constant at the front.
                return Self::Add(vec![&[Expr::from(c)],n.split_at(cutoff).1].concat());
            },
            Self::Mul(n) => {
                // Reduce const for all items
                let mut n: Vec<Expr> = n.iter().map(|a| a.reduce_const()).collect();

                // Sort the items so constants are first, then find the cutoff where the items are no longer 
                // constant.
                n.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                let mut cutoff = usize::MAX;
                for i in 0..n.len() {
                    if !n[i].is_const() {break}
                    cutoff = i+1;
                }

                println!("   - simplify {n:?}");

                // No constants found
                if cutoff == usize::MAX {
                    return Self::Mul(n);
                }

                // Accumulate all of the constant values
                let c = n[1..cutoff].iter().fold(n[0].force_const(),|acc, a| acc * a.force_const());

                // If all of the values were constant, return a term.
                if cutoff == n.len() {
                    return *f::term(c);
                }

                // Return the same terms with the reduced constant at the front.
                return Self::Mul(vec![&[Expr::from(c)],n.split_at(cutoff).1].concat());
            },
            Self::Pow(a, b) => {
                let a = a.reduce_const();
                let b = b.reduce_const();
                if a.is_const() & b.is_const() {
                    return (a.force_const().pow(b.force_const())).into();
                }
                Self::Pow(a.r#box(), b.r#box())
            },
            Self::Fn(_, _) => self.clone(),
        }
    }

    /// Reorders some add and mul operations to put constant terms first.
    /// This aids in other simplification processes.
    pub fn reorder(&self) -> Self {
        match self {
            Self::Add(n) => {
                let mut n = n.clone();
                n.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                return Self::Add(n.to_vec());
            },
            Self::Mul(n) => {
                let mut n = n.clone();
                n.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                return Self::Mul(n.to_vec());
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
            Self::Add(n) => {
                if n[0].is_zero() { return Self::Add(n.split_at(1).1.to_vec()) } // 0 + n
                return self.clone();
            },
            Self::Mul(n) => {
                if n[0].is_one() { return Self::Mul(n.split_at(1).1.to_vec()) } // 1 * n
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
    pub fn expand_pow(&self) -> Self {
        match self {
            Self::Pow(a, b) => {
                if !b.is_const() {return self.clone();}
                let b = b.force_const();
                if b == Term::from(2.0) { return Self::Mul(vec![*a.clone(); 2]); }
                if b == Term::from(3.0) { return Self::Mul(vec![*a.clone(); 3]); }
                if b == Term::from(4.0) { return Self::Mul(vec![*a.clone(); 4]); }
                if b == Term::from(5.0) { return Self::Mul(vec![*a.clone(); 5]); }
                return self.clone();
            },
            Self::Add(n) => Self::Add(n.iter().map(|a| a.expand_pow()).collect()),
            Self::Mul(n) => Self::Mul(n.iter().map(|a| a.expand_pow()).collect()),
            Self::Fn(_, _) => self.clone(),
            Self::Term(_) => self.clone(),
        }
    }

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
            Self::Add(n) => n.iter().map(|a| a.has_var(var, c)).collect::<Vec<bool>>().contains(&true),
            Self::Mul(n) => n.iter().map(|a| a.has_var(var, c)).collect::<Vec<bool>>().contains(&true),
            Self::Pow(a, b) => a.has_var(var, c) | b.has_var(var, c),
            Self::Fn(_, a) => a.iter().map(|a| a.has_var(var, c)).collect::<Vec<bool>>().contains(&true),
        }
    }

    /// Checks if this expression tree contains a function.
    /// Search includes other functions in a `Context`.
    pub fn has_fn(&self, name: &String, c: &Context) -> bool {
        match self {
            Self::Term(_) => false,
            Self::Add(n) => n.iter().map(|a| a.has_fn(name, c)).collect::<Vec<bool>>().contains(&true),
            Self::Mul(n) => n.iter().map(|a| a.has_fn(name, c)).collect::<Vec<bool>>().contains(&true),
            Self::Pow(a, b) => a.has_fn(name, c) | b.has_fn(name, c),
            Self::Fn(_, _) => true,
        }
    }

    /// Checks if `self` is negative
    pub fn is_neg(&self) -> bool {
        match self {
            Self::Mul(n) => n[0].is_neg_one(),
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
    /// Checks if `self` is a commutative operation.
    pub fn is_commutative(&self) -> bool {
        match self {
            Self::Add(_,) | Self::Mul(_,) => true,
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
    /// Checks if `self` is a constant term.
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

    pub fn order_num(&self) -> u8 {
        match self {
            Self::Term(Term::Real(_)) => 0,
            Self::Term(Term::Complex(_)) => 0,
            Self::Term(Term::Var(_)) => 1,
            _ => 2,
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
impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order_num().partial_cmp(&other.order_num())
    }
}


/// A term in the expression tree.
#[derive(Debug, Clone)]
pub enum Term {
    Real(f64),
    Complex(Complex64),
    Var(String),
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
                Self::Var(n) => {if n == var {t = (*val).expand_vars(vars).r#box()}},
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
/// Orders terms based on how they should be ordered in expressions. (less -> more)
impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Term::Real(_), Term::Real(_)) => Some(std::cmp::Ordering::Equal),
            (Term::Real(_), Term::Complex(_)) => Some(std::cmp::Ordering::Equal),
            (Term::Real(_), Term::Var(_)) => Some(std::cmp::Ordering::Less),
            (Term::Complex(_), Term::Real(_)) => Some(std::cmp::Ordering::Equal),
            (Term::Complex(_), Term::Complex(_)) => Some(std::cmp::Ordering::Equal),
            (Term::Complex(_), Term::Var(_)) => Some(std::cmp::Ordering::Less),
            (Term::Var(_), Term::Real(_)) => Some(std::cmp::Ordering::Greater),
            (Term::Var(_), Term::Complex(_)) => Some(std::cmp::Ordering::Greater),
            (Term::Var(a), Term::Var(b)) => a.partial_cmp(b),
        }
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Real(a), Self::Real(b)) => a == b,
            (Self::Real(a), Self::Complex(b)) => Complex64::from(a) == *b,
            (Self::Complex(a), Self::Real(b)) => a == &Complex64::from(b),
            (Self::Complex(a), Self::Complex(b)) => a == b,
            (Self::Var(a), Self::Var(b)) => a == b,
            _ => false,
        }
    }
}