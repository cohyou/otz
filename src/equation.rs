use std::rc::Rc;

use crate::context::Context;
use crate::symbol_table::Names;
use crate::term::{Term, TermInner};

#[derive(PartialEq, Clone)]
pub struct Equation {
    pub context: Rc<Context>,
    pub names: Rc<Names>,
    pub left: Rc<TermInner>,
    pub right: Rc<TermInner>,
}

impl std::fmt::Debug for Equation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} | {:?} = {:?}",
            self.context.0, self.left, self.right
        )
    }
}

impl std::fmt::Display for Equation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Eq<{} | {} = {}>",
            self.context,
            self.left_term(),
            self.right_term()
        )
    }
}

impl Equation {
    pub fn left_term(&self) -> Term {
        Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.left.clone(),
        }
    }
    pub fn right_term(&self) -> Term {
        Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.right.clone(),
        }
    }
}
