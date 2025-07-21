use crate::id::{OperId, TypeId};
type Link<T> = std::rc::Rc<T>;

#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    Unary(TypeId),
    Binary(OperId, Link<Type>, Link<Type>),
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unary(id) => write!(f, "Type{:?}", id.0),
            Type::Binary(op_id, dom, cod) => write!(f, "Type({:?} {:?}.{:?})", op_id, dom, cod),
        }
    }
}
