use crate::id::{OperId, TypeId};
use crate::oper::Oper;
use crate::parser::oper::oper_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::symbol_table::SymbolTable;
use combine::parser::char;
use combine::parser::char::spaces;
use combine::stream::Stream;
use combine::Parser;

/// 型名
pub fn fkey_decl_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Oper> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    char::string(DIRECTIVE_SIGN)
        .and(char::string("fkey"))
        .and(spaces())
        .with(oper_parser(types, opers))
}

#[test]
fn test_fkey_decl_parser() {
    use crate::combine::EasyParser;

    let input = "#fkey mgr: Emp -> Emp";

    let opers = SymbolTable::<OperId>::new();
    opers.assign("mgr".to_string());
    let types = SymbolTable::<TypeId>::new();
    types.assign("Emp".to_string());
    let result = fkey_decl_parser(&opers, &types).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
