use crate::id::{OperId, VarId};
type Link<T> = std::rc::Rc<T>;

#[derive(PartialEq, Clone)]
pub enum TermInner {
    Var(VarId),
    Fun(OperId, Vec<Link<TermInner>>),
}

impl std::fmt::Debug for TermInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermInner::Var(id) => write!(f, "Var{:?}", id.0),
            TermInner::Fun(op_id, args) => write!(f, "Fun{:?}{:?}", op_id.0, args),
        }
    }
}
