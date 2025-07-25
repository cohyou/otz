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
pub fn oper_decl_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Oper> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    char::string(DIRECTIVE_SIGN)
        .and(char::string("func"))
        .and(spaces())
        .with(oper_parser(opers, types))
}

#[test]
fn test_oper_decl_parser() {
    use crate::combine::EasyParser;

    let type_name_example = "#sort Bool";

    let opers = SymbolTable::<OperId>::init_with(OperId(2));
    let types = SymbolTable::<TypeId>::init_with(TypeId(2));
    let _r = oper_decl_parser(&opers, &types).easy_parse(type_name_example);
    dbg!(&opers);
    assert_eq!(types.get("Bool"), Some(TypeId(2)));
}
