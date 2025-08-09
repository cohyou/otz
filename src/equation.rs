use std::rc::Rc;

use crate::context::Context;
use crate::term::{Term, TermInner};
#[derive(PartialEq, Clone)]
pub struct Equation {
    pub context: Context,
    pub left: Rc<TermInner>,
    pub right: Rc<TermInner>,
}

impl std::fmt::Debug for Equation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} | {:?} = {:?}", self.context.0, self.left, self.right)
    }
}

impl Equation {
    pub fn left_term(&self) -> Term {
        Term {
            context: self.context.clone(),
            inner: self.left.clone(),
        }
    }
    pub fn right_term(&self) -> Term {
        Term {
            context: self.context.clone(),
            inner: self.right.clone(),
        }
    }
}