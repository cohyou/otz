use crate::equation::Equation;
use crate::oper::Oper;
use crate::r#type::Type;
use crate::theory::Theory;

#[derive(Default, Clone, Debug)]
pub struct Schema {
    pub theory: Theory,
    pub entities: Vec<Type>,
    pub fkeys: Vec<Oper>,
    pub attrs: Vec<Oper>,
    pub constraints: Vec<Equation>,
}
