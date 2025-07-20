use crate::id::{OperId, TypeId};
type Link<T> = std::rc::Rc<T>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Unary(TypeId),
    Binary(OperId, Link<Type>, Link<Type>),
}
