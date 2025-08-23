use std::rc::Rc;

use crate::term::Term;

#[derive(Clone, PartialEq)]
pub struct Rule {
    pub before: Rc<Term>,
    pub after: Rc<Term>,
}
