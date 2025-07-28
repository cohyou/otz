use crate::context::Context;
use crate::term::TermInner;
#[derive(PartialEq, Clone)]
pub struct Equation {
    pub context: Context,
    pub left: TermInner,
    pub right: TermInner,
}

impl std::fmt::Debug for Equation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} | {:?} = {:?}", self.context.0, self.left, self.right)
    }
}
