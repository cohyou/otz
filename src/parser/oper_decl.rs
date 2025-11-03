use crate::id::{OperId, TypeId};
use crate::oper::Oper;
use crate::parser::oper::oper_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::symbol_table::SymbolTable;
use combine::parser::char;
use combine::parser::char::spaces;
use combine::stream::Stream;
use combine::Parser;

pub fn oper_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Oper> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    char::string(DIRECTIVE_SIGN)
        .and(char::string("func"))
        .and(spaces())
        .with(oper_parser(types, opers))
}

#[test]
fn test_oper_decl_parser() {
    use crate::combine::EasyParser;

    let input = "#func not: Bool -> Bool";

    let opers = SymbolTable::<OperId>::new();
    opers.assign("not".to_string());
    let types = SymbolTable::<TypeId>::new();
    types.assign("Bool".to_string());

    let result = oper_decl_parser(&types, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
