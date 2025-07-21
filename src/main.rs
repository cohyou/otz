extern crate combine;

mod context;
mod equation;
mod id;
mod oper;
mod term;
mod theory;
mod r#type;

pub mod schema;
pub mod symbol_table;
pub mod context_table;
pub mod parser;

fn main() {
    use crate::parser::parse_schema;

    let path = "schema/s.schema";
    let schema = parse_schema(path);
    dbg!(schema);
}
