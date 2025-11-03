use crate::id::{OperId, TypeId};
use crate::oper::Oper;
use crate::parser::oper::oper_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::symbol_table::SymbolTable;
use combine::parser::char::{spaces, string};

use combine::stream::Stream;
use combine::Parser;

/// 型名
pub fn attr_decl_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Oper> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string(DIRECTIVE_SIGN)
        .and(string("attr"))
        .and(spaces())
        .with(oper_parser(types, opers))
}

#[test]
fn test_attr_decl_parser() {
    use combine::EasyParser;

    let input = "#attr name: Student -> Str";

    let opers = SymbolTable::<OperId>::new();
    opers.assign("name".to_string());
    let types = SymbolTable::<TypeId>::new();
    types.assign("Student".to_string());
    types.assign("Str".to_string());
    let result = attr_decl_parser(&opers, &types).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
