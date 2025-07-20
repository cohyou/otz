use crate::context::Ctxt;
use crate::term::TermInner;

#[derive(Debug, PartialEq, Clone)]
pub struct Equation {
    pub context: Ctxt,
    pub left: TermInner,
    pub right: TermInner,
}
