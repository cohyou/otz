mod attr_decl;
pub mod context;
mod eq_decl;
pub mod equation;
mod fkey_decl;
pub mod oper;
mod oper_decl;
mod schema;
pub mod term;
mod theory;
mod theory_decl;
pub mod r#type;
mod type_decl;
mod variable;

pub const DIRECTIVE_SIGN: &'static str = "#";

pub fn parse_schema(path: &str) -> crate::schema::Schema {
    use crate::combine::EasyParser;
    use crate::id::{TypeId, OperId};
    use crate::symbol_table::SymbolTable;
    use crate::context_table::CtxtTable;
    use crate::parser::schema::schema_parser;

    let schema_example = std::fs::read_to_string(path).expect("Failed to read");

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();
    let result = schema_parser::<combine::easy::Stream<&str>>(&types, &opers, &ctxts)
        .easy_parse(schema_example.as_ref());
    result.unwrap().0
}