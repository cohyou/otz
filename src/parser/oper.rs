use std::rc::Rc;

use combine::parser::char::{alpha_num, spaces, string};
use combine::stream::Stream;
use combine::{many1, Parser};

use crate::id::{OperId, TypeId};
use crate::oper::Oper;
use crate::parser::r#type::type_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::symbol_table::SymbolTable;

pub fn oper_parser<'a, Input>(
    table: &'a SymbolTable<OperId>,
    type_table: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Oper> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string(DIRECTIVE_SIGN)
        .and(string("func"))
        .and(spaces())
        .with(many1(alpha_num()))
        .skip(spaces())
        .and(string(":"))
        .skip(spaces())
        .and(type_parser(type_table, table))
        .skip(spaces())
        .skip(string("->"))
        .skip(spaces())
        .and(type_parser(type_table, table))
        .map(move |(((c, _), dom), cod): (((Vec<_>, _), _), _)| {
            let name: String = c.into_iter().collect();
            dbg!(&dom, &cod);
            let id = table.assign(name);
            let dom = Rc::new(dom);
            let cod = Rc::new(cod);
            Oper::new(id.clone(), dom, cod)
        })
}

#[test]
fn test_oper_parser() {
    use crate::combine::EasyParser;

    let type_name_example = "#func not: Bool -> Bool";

    let table = SymbolTable::<OperId>::init_with(OperId(3));
    let type_table = SymbolTable::<TypeId>::init_with(TypeId(2));
    type_table.insert("Bool".to_string(), TypeId(2));
    let _r = oper_parser(&table, &type_table).easy_parse(type_name_example);
    dbg!(&table);
    assert_eq!(table.get("not"), Some(OperId(3)));
}

#[test]
fn test_oper_parser_binary_type() {
    use crate::combine::EasyParser;

    let type_name_example = "#func and: Bool * Bool -> Bool";

    let table = SymbolTable::<OperId>::init_with(OperId(2));
    let type_table = SymbolTable::<TypeId>::init_with(TypeId(2));
    type_table.insert("Bool".to_string(), TypeId(2));
    let _r = oper_parser(&table, &type_table).easy_parse(type_name_example);
    dbg!(&table);
    assert_eq!(table.get("and"), Some(OperId(3)));
}
