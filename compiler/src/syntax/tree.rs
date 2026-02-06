extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use crate::spine::Literal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxExpr {
    Root(Vec<SyntaxExpr>),
    Def {
        name: String,
        binders: Vec<Binder>,
        return_type: Box<SyntaxExpr>,
        body: Box<SyntaxExpr>,
    },
    Var(String),
    Constructor(String),
    App(Box<SyntaxExpr>, Box<SyntaxExpr>),
    Lambda {
        binders: Vec<Binder>,
        body: Box<SyntaxExpr>,
    },
    Let {
        name: String,
        type_ann: Option<Box<SyntaxExpr>>,
        value: Box<SyntaxExpr>,
        body: Box<SyntaxExpr>,
    },
    Lit(Literal),
    Tuple(Vec<SyntaxExpr>),
    Proj(Box<SyntaxExpr>, String),
    Hole,
    Arrow(Box<SyntaxExpr>, Box<SyntaxExpr>),
    List(Box<SyntaxExpr>),
    Pi(Binder, Box<SyntaxExpr>),
    Sigma(Binder, Box<SyntaxExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binder {
    Explicit(String, Box<SyntaxExpr>),
    Implicit(String, Box<SyntaxExpr>),
    Instance(String, Box<SyntaxExpr>),
}
