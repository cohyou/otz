use crate::id::{OperId, VarId};
type Link<T> = std::rc::Rc<T>;

#[derive(Debug, PartialEq, Clone)]
pub enum TermInner {
    Var(VarId),
    Fun(OperId, Vec<Link<TermInner>>),
}
