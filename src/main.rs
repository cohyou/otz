extern crate combine;

mod context;
mod equation;
mod id;
mod oper;
mod term;
mod theory;
mod r#type;
mod saturate;
mod subst;

pub mod context_table;
pub mod instance;
pub mod parser;
pub mod schema;
pub mod symbol_table;
pub mod eval;

fn main() {
    use crate::parser::parse_instance;
    let path = "instance/i.instance";
    let instance = parse_instance(path);
    dbg!(&instance);

    use crate::eval::eval;
    let queried = eval(instance);
    dbg!(&queried);
}
