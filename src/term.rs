use std::rc::Rc;

use crate::{context::Context, id::{OperId, VarId}};
type Link<T> = std::rc::Rc<T>;

#[derive(PartialEq, Clone)]
pub enum TermInner {
    Var(VarId),
    Fun(OperId, Vec<Link<TermInner>>),
    Str(String),
    Int(usize),
}

impl std::fmt::Debug for TermInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermInner::Var(id) => write!(f, "Var{:?}", id.0),
            TermInner::Fun(op_id, args) => write!(f, "Fun{:?}{:?}", op_id.0, args),
            TermInner::Str(s) => write!(f, "Str{:?}", s),
            TermInner::Int(i) => write!(f, "Int{:?}", i),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Term {
    pub context: Context,
    pub inner: Rc<TermInner>,
}
