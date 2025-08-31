use std::rc::Rc;

use crate::{
    context::Context,
    term::{Term, TermInner},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Rule {
    pub context: Context,
    before: Rc<TermInner>,
    after: Rc<TermInner>,
}

impl Rule {
    pub fn new(context: Context, before: Rc<TermInner>, after: Rc<TermInner>) -> Self {
        Rule {
            context,
            before,
            after,
        }
    }
    pub fn before(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(),
            inner: self.before.clone(),
        })
    }

    pub fn after(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(),
            inner: self.after.clone(),
        })
    }
}
