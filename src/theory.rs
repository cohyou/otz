use crate::r#type::Type;
use crate::oper::Oper;
use crate::equation::Equation;

#[derive(Default, Clone, Debug)]
pub struct Theory {
    pub types: Vec<Type>,
    pub opers: Vec<Oper>,
    pub eqs: Vec<Equation>,
}
