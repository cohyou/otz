extern crate combine;

mod context;
mod equation;
mod id;
mod oper;
mod term;
mod theory;
mod r#type;

pub mod context_table;
pub mod instance;
pub mod parser;
pub mod schema;
pub mod symbol_table;

fn main() {
    use crate::parser::parse_instance;
    use crate::parser::parse_schema;

    let path = "schema/s.schema";
    let schema = parse_schema(path);
    dbg!(schema);

    let path = "instance/i.instance";
    let instance = parse_instance(path);
    dbg!(instance);
}
