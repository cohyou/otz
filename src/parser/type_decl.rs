use crate::id::TypeId;
use crate::parser::r#type::type_unary_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::r#type::Type;
use crate::symbol_table::SymbolTable;
use combine::parser::char;
use combine::parser::char::spaces;
use combine::stream::Stream;
use combine::Parser;

/// 型名
pub fn type_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Type> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    char::string(DIRECTIVE_SIGN)
        .and(char::string("sort"))
        .and(spaces())
        .with(type_unary_parser(types))
}

#[test]
fn test_type_decl_parser() {
    use crate::combine::EasyParser;

    let type_name_example = "#sort Bool";

    let table = SymbolTable::<TypeId>::init_with(TypeId(2));
    let _r = type_decl_parser(&table).easy_parse(type_name_example);
    dbg!(&table);
    assert_eq!(table.get("Bool"), Some(TypeId(2)));
}
