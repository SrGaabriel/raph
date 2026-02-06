use alloc::{boxed::Box, string::String};

use crate::{module::unique::Unique};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    BVar(usize),
    FVar(Unique),
    MVar(Unique),
    App(Box<Term>, Box<Term>),
    Sort(Level),
    Lam(BinderInfo, Box<Term>, Box<Term>),
    Pi(BinderInfo, Box<Term>, Box<Term>),
    Sigma(BinderInfo, Box<Term>, Box<Term>),
    Let(Box<Term>, Box<Term>, Box<Term>),
    Lit(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinderInfo {
    Explicit,
    Implicit,
    InstanceImplicit,
    StrictImplicit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Nat(u64),
    Str(String),    
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Zero,
    Succ(Box<Level>),
    Max(Box<Level>, Box<Level>),
    IMax(Box<Level>, Box<Level>),
    MVar(Unique),
}