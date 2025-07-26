use crate::{context::Ctxt, equation::Equation, schema::Schema};

#[derive(Default, Clone, Debug)]
pub struct Instance {
    pub schema: Schema,
    pub elems: Ctxt,
    pub data: Vec<Equation>,
}
