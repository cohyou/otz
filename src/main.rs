extern crate combine;

mod analyse;
mod completion;
mod context;
mod equation;
mod id;
mod oper;
mod overlap;
mod reduct;
mod rule;
mod saturate;
mod subst;
mod subterm;
mod term;
mod theory;
mod r#type;
mod unify;

pub mod context_table;
pub mod eval;
pub mod instance;
pub mod parser;
pub mod schema;
pub mod symbol_table;
pub mod util;

fn main() {
    use crate::parser::parse_instance;
    let path = "instance/i.instance";
    let instance = parse_instance(path);
    dbg!(&instance);

    use crate::eval::eval;
    use crate::eval::Query;
    let queried = eval(instance, Query::default());
    dbg!(&queried);
}
