use autoincrement::prelude::*;

#[derive(AsyncIncremental, PartialEq, Eq, Clone, Default)]
pub struct TypeId(pub usize);

#[derive(AsyncIncremental, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct OperId(pub usize);

#[derive(Default, Hash, AsyncIncremental, PartialEq, Eq, Clone)]
pub struct CtxtId(pub usize);
#[derive(Hash, AsyncIncremental, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct VarId(pub usize);

impl std::fmt::Debug for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type{}", self.0)
    }
}

impl std::fmt::Debug for OperId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Oper{}", self.0)
    }
}

impl std::fmt::Debug for CtxtId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ctxt{}", self.0)
    }
}

impl std::fmt::Debug for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Var{}", self.0)
    }
}
