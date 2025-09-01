use std::rc::Rc;

use crate::{
    context::Context,
    id::{OperId, VarId}, rule::{RuleId, RuleKind},
};
type Link<T> = std::rc::Rc<T>;

#[derive(PartialEq, Clone)]
pub enum TermInner {
    Var(VarId),
    Fun(OperId, Vec<Link<TermInner>>),
    Str(String),
    Int(usize),

    RuledVar(VarId, RuleId, RuleKind),

    Subst(VarId, Rc<TermInner>),
}

impl std::fmt::Debug for TermInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermInner::Var(id) => write!(f, "Var{:?}", id.0),
            TermInner::Fun(op_id, args) => write!(f, "Fun{:?}{:?}", op_id.0, args),
            TermInner::Str(s) => write!(f, "Str{:?}", s),
            TermInner::Int(i) => write!(f, "Int{:?}", i),

            TermInner::RuledVar(vid, rid, kind) => write!(f, "Var<{:?},{:?},{:?}>", vid.0, rid, kind),

            TermInner::Subst(varid, inner) => write!(f, "Subst[{:?}->{:?}]", varid, inner),
        }
    }
}
#[derive(Clone, PartialEq)]
pub struct Term {
    pub context: Context,
    pub inner: Rc<TermInner>,
}

impl std::fmt::Debug for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
