use std::rc::Rc;

use crate::context::Context;
use crate::symbol_table::Names;
use crate::term::{Term, TermInner};

#[derive(Clone)]
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
            "Eq<{} = {}>",
            // self.context,
            self.left_term(),
            self.right_term()
        )
    }
}

impl std::cmp::PartialEq for Equation {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right
    }
}

impl std::cmp::Eq for Equation {}

impl std::cmp::PartialOrd for Equation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (std::cmp::max(other.left.var_size(), other.right.var_size()))
            .partial_cmp(std::cmp::max(&self.left.var_size(), &self.right.var_size()))
    }
}

impl std::cmp::Ord for Equation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
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
