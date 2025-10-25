use crate::id::OperId;
use crate::r#type::Type;
type Link<T> = std::rc::Rc<T>;

#[derive(PartialEq, Eq, Clone)]
pub struct Oper {
    pub id: OperId,
    dom: Link<Type>,
    pub cod: Link<Type>,
}

impl Oper {
    pub fn new(id: OperId, dom: Link<Type>, cod: Link<Type>) -> Self {
        Oper {
            id,
            dom: dom,
            cod: cod,
        }
    }
}

impl std::fmt::Debug for Oper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Oper{:?}: {:?} -> {:?}", self.id.0, self.dom, self.cod)
    }
}
