
/*
This is where the data for the parsel parser is.
*/


use parsel::{
    self, ast::{Brace, LeftAssoc, LitFloat, LitInt, Many, Paren, Punctuated}, parse_str, syn::{token::{Caret, Comma, Eq, Fn, Let, Minus, Plus, Semi, Slash, Star}, Ident, Token}, Parse, ToTokens
};

// Custom keywords
mod kw {
    use parsel::custom_keyword;

    custom_keyword!(sin);
    custom_keyword!(cos);
    custom_keyword!(tan);
    custom_keyword!(sinh);
    custom_keyword!(cosh);
    custom_keyword!(tanh);
    custom_keyword!(asin);
    custom_keyword!(acos);
    custom_keyword!(atan);
    custom_keyword!(asinh);
    custom_keyword!(acosh);
    custom_keyword!(atanh);

    custom_keyword!(ln);
    custom_keyword!(log);

    custom_keyword!(root);
    custom_keyword!(sqrt);
    custom_keyword!(cbrt);


}

pub fn str_parse(s: &str) -> Vec<Statement> {
    parse_str::<Many<Statement>>(s).unwrap().into_iter().collect()
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
/// A standalone statement in some context description
pub enum Statement {
    Let{
        kw_let: Let,
        name: Ident,
        kw_eq: Eq,
        body: Expr,
        kw_semi: Semi,
    },
    Fn {
        kw_fn: Fn,
        name: Ident,
        args: Paren<Punctuated<Ident, Comma>>,
        kw_eq: Eq,
        body: Expr,
        kw_semi: Semi,
    },
}


#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
/// A right-associated node of an expression tree.
pub enum Node {
    
    Ln(kw::ln, #[parsel(recursive)] Paren<Box<Expr>>),
    Log(kw::log, #[parsel(recursive)] Brace<Box<Expr>>, #[parsel(recursive)] Paren<Box<Expr>>),
    
    Root(kw::log, #[parsel(recursive)] Brace<Box<Expr>>, #[parsel(recursive)] Paren<Box<Expr>>),
    Sqrt(kw::sqrt, #[parsel(recursive)] Paren<Box<Expr>>),
    Cbrt(kw::cbrt, #[parsel(recursive)] Paren<Box<Expr>>),
    
    Sin(kw::sin, #[parsel(recursive)] Paren<Box<Expr>>),
    Cos(kw::cos, #[parsel(recursive)] Paren<Box<Expr>>),
    Tan(kw::tan, #[parsel(recursive)] Paren<Box<Expr>>),
    Sinh(kw::sinh, #[parsel(recursive)] Paren<Box<Expr>>),
    Cosh(kw::cosh, #[parsel(recursive)] Paren<Box<Expr>>),
    Tanh(kw::tanh, #[parsel(recursive)] Paren<Box<Expr>>),
    Asin(kw::asin, #[parsel(recursive)] Paren<Box<Expr>>),
    Acos(kw::acos, #[parsel(recursive)] Paren<Box<Expr>>),
    Atan(kw::atan, #[parsel(recursive)] Paren<Box<Expr>>),
    Asinh(kw::asinh, #[parsel(recursive)] Paren<Box<Expr>>),
    Acosh(kw::acosh, #[parsel(recursive)] Paren<Box<Expr>>),
    Atanh(kw::atanh, #[parsel(recursive)] Paren<Box<Expr>>),

    
    Abs(Token![|], #[parsel(recursive)] Box<Expr>, Token![|]),
    
    Fn(Ident, #[parsel(recursive)] Paren<Punctuated<Expr, Comma>>),
    Paren(#[parsel(recursive)] Paren<Box<Expr>>),
    
    Term(Term),
    Neg(Minus, #[parsel(recursive)] Box<Expr>),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
pub enum Term {
    Var(Ident),
    Float(LitFloat),
    Int(LitInt),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
pub enum AddOp {
    Add(Plus),
    Sub(Minus),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
pub enum MulOp {
    Mul(Star),
    Div(Slash),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
pub enum PowOp {
    Pow(Caret),
}

/// The expression type. Made to include left- and right-associated features.
pub type Expr = LeftAssoc<
    AddOp,
    LeftAssoc<
        MulOp,
        LeftAssoc<
            PowOp,
            Node,
        >
    >
>;