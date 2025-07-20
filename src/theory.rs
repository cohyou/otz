use crate::equation::Equation;
use crate::oper::Oper;
use crate::r#type::Type;

#[derive(Default, Clone, Debug)]
pub struct Theory {
    pub types: Vec<Type>,
    pub opers: Vec<Oper>,
    pub eqs: Vec<Equation>,
}
