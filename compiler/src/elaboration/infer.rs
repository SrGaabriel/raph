use crate::{
    elaboration::ElabState,
    module::prim::{prim_nat, prim_string},
    spine::{Literal, Term},
};

pub fn infer_type(state: &mut ElabState, term: Term) -> Option<Term> {
    match term {
        Term::Lit(Literal::Nat(_)) => Some(Term::Const(prim_nat())),
        Term::Lit(Literal::Str(_)) => Some(Term::Const(prim_string())),
        Term::App(fun, _arg) => {
            let fun_type = infer_type(state, *fun)?;
            match fun_type {
                Term::Pi(_info, _param_ty, body_ty) => Some(*body_ty),
                Term::Lam(_info, _param_ty, body_ty) => Some(*body_ty),
                _ => None,
            }
        }
        Term::Const(name) => {
            let decl = state.env.lookup(&name)?;
            Some(decl.type_().clone())
        }
        _ => todo!(),
    }
}
