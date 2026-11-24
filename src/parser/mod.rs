mod attr_decl;
pub mod context;
mod data_decl;
mod elem_decl;
mod eq_decl;
pub mod equation;
mod fkey_decl;
mod instance;
pub mod oper;
mod oper_decl;
pub mod rule;
mod schema;
mod schema_decl;
pub mod term;
mod theory;
mod theory_decl;
pub mod r#type;
mod type_decl;
mod variable;
mod elem;
mod elems;
pub mod query;
mod instance_decl;

pub const DIRECTIVE_SIGN: &'static str = "#";

pub fn parse_schema(path: &str) -> crate::schema::Schema {
    use crate::combine::EasyParser;
    use crate::context_table::CtxtTable;
    use crate::id::{OperId, TypeId};
    use crate::parser::schema::schema_parser;
    use crate::symbol_table::SymbolTable;

    let schema_example = std::fs::read_to_string(path).expect("Failed to read");

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();
    let result = schema_parser::<combine::easy::Stream<&str>>(&types, &opers, &ctxts)
        .easy_parse(schema_example.as_ref());
    result.unwrap().0
}

pub fn parse_instance(path: &str) -> crate::instance::Instance {
    use crate::context_table::CtxtTable;
    use crate::id::{OperId, TypeId};
    use crate::symbol_table::SymbolTable;

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();
    parse_instance_with(path, &types, &opers, &ctxts)
}

/// 呼び出し側の記号表を使ってインスタンスを読み込む。
/// 同じ記号表で後からクエリなどを読めば、要素や演算子の ID がインスタンスと一致する。
pub fn parse_instance_with(
    path: &str,
    types: &crate::symbol_table::SymbolTable<crate::id::TypeId>,
    opers: &crate::symbol_table::SymbolTable<crate::id::OperId>,
    ctxts: &crate::context_table::CtxtTable,
) -> crate::instance::Instance {
    use crate::combine::EasyParser;
    use crate::parser::instance::instance_parser;

    let instance_src = std::fs::read_to_string(path).expect("Failed to read");
    let result = instance_parser::<combine::easy::Stream<&str>>(types, opers, ctxts)
        .easy_parse(instance_src.as_ref());
    result.unwrap().0
}
