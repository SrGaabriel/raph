use alloc::{collections::btree_map::BTreeMap, vec::Vec};

use crate::{module::unique::{Unique, UniqueGen}, spine::Term};

#[derive(Debug, Clone)]
pub struct LocalDecl {
    fvar: Unique,
    type_: Term,
    value: Option<Term>,
}

#[derive(Debug, Clone)]
pub struct LocalContext {
    decls: Vec<LocalDecl>,
}

#[derive(Debug, Clone)]
pub struct MetavarDecl {
    pub mvar: Unique,
    pub type_: Term,
    pub lctx: LocalContext,
}

#[derive(Debug)]
pub struct MetavarContext {
    pub decls: Vec<MetavarDecl>,
    pub assignments: BTreeMap<Unique, Term>,
}

impl MetavarContext {
    pub fn new() -> Self {
        Self {
            decls: Vec::new(),
            assignments: BTreeMap::new(),
        }
    }

    pub fn fresh_mvar(&mut self, type_: Term, lctx: &LocalContext, gen_: &mut UniqueGen) -> Unique {
        let mvar = gen_.fresh_unnamed();
        self.decls.push(MetavarDecl {
            mvar: mvar.clone(),
            type_,
            lctx: lctx.clone(),
        });
        mvar
    }

    pub fn assign(&mut self, mvar: Unique, value: Term) {
        assert!(!self.assignments.contains_key(&mvar), "mvar already assigned");
        self.assignments.insert(mvar, value);
    }

    pub fn is_assigned(&self, mvar: Unique) -> bool {
        self.assignments.contains_key(&mvar)
    }

    pub fn get_assignment(&self, mvar: Unique) -> Option<&Term> {
        self.assignments.get(&mvar)
    }

    pub fn lookup_decl(&self, mvar: Unique) -> Option<&MetavarDecl> {
        self.decls.iter().find(|d| d.mvar == mvar)
    }
}