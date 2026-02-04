extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

pub trait Phase {
    type ExprAnn;
    type TyAnn;
}

pub struct Parsed;
pub struct Typed;

impl Phase for Parsed {
    type ExprAnn = ();
    type TyAnn = ();
}

impl Phase for Typed {
    type ExprAnn = ();
    type TyAnn = ();
}

pub struct Expr<P: Phase> {
    pub ann: P::ExprAnn,
    pub kind: ExprKind<P>,
}

pub enum ExprKind<P: Phase> {
    Var(String),
    App(Box<Expr<P>>, Box<Expr<P>>),
}
