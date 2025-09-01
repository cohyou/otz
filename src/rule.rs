use std::rc::Rc;

use crate::{
    context::Context,
    term::{Term, TermInner},
};

pub type RuleId = usize;
#[derive(Clone, PartialEq, Debug, Hash, Eq, PartialOrd, Ord)]
pub enum RuleKind { NotSet, Set1, Set2 }

#[derive(Clone, PartialEq, Debug)]
pub struct Rule {
    // pub kind: RuleKind,
    pub id: Option<RuleId>,
    pub context: Context,
    pub before: Rc<TermInner>,
    pub after: Rc<TermInner>,
}

impl Rule {
    pub fn new(context: Context, before: Rc<TermInner>, after: Rc<TermInner>) -> Self {
        Rule {
            // kind: RuleKind::NotSet,
            id: None,
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
