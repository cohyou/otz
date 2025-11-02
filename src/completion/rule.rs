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
    pub context: Rc<Context>,
    pub names: Rc<Names>,
    pub before: Rc<TermInner>,
    pub after: Rc<TermInner>,
}

impl Rule {
    pub fn new(
        context: Rc<Context>,
        names: Rc<Names>,
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

impl std::cmp::PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (other.before.size() + other.after.size())
            .partial_cmp(&(self.before.size() + self.after.size()))
    }
}
impl std::cmp::Ord for Rule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::cmp::Eq for Rule {}
