use std::rc::Rc;

use crate::{
    context::Context,
    symbol_table::Names,
    term::{Term, TermInner},
};

pub type RuleId = usize;
#[derive(Clone, PartialEq, Debug, Hash, Eq, PartialOrd, Ord)]
pub enum RuleKind {
    NotSet,
    Set1,
    Set2,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Rule {
    pub id: Option<RuleId>,
    pub context: Context,
    pub names: Names,
    pub before: Rc<TermInner>,
    pub after: Rc<TermInner>,
}

impl Rule {
    pub fn new(
        context: Context,
        names: Names,
        before: Rc<TermInner>,
        after: Rc<TermInner>,
    ) -> Self {
        Rule {
            id: None,
            names,
            context,
            before,
            after,
        }
    }

    pub fn before(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.before.clone(),
        })
    }

    pub fn after(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.after.clone(),
        })
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rule< {} -> {} >", self.before(), self.after())
    }
}
