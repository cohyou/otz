use crate::combine::{many1, Parser};
use crate::id::{OperId, TypeId};
use crate::r#type::Type;
use crate::symbol_table::SymbolTable;
use combine::parser::char::spaces;
use combine::parser::char::{self};
use combine::stream::Stream;
use combine::{attempt, stream};
use std::rc::Rc;

pub fn type_parser<'a, Input>(
    table: &'a SymbolTable<TypeId>,
    oper_table: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Type> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let type_unary_parser = type_unary_parser(table);
    let type_binary_parser = type_binary_parser(table, oper_table);

    attempt(type_binary_parser).or(type_unary_parser)
}

#[test]
fn test_type_parser_unary() {
    use crate::combine::EasyParser;

    let type_name_example = "Bool";

    let table = SymbolTable::<TypeId>::init_with(TypeId(3));
    let oper_table = SymbolTable::<OperId>::new();
    let r = type_parser(&table, &oper_table).easy_parse(type_name_example);
    assert_eq!(r, Ok((Type::Unary(TypeId(3)), "")));
    dbg!(&table);
    assert_eq!(table.get("Bool"), Some(TypeId(3)));
}

#[test]
fn test_type_parser_binary() {
    use crate::combine::EasyParser;
    use std::rc::Rc;

    let type_name_example = "Bool * Bool";
    let table = SymbolTable::<TypeId>::init_with(TypeId(2));
    let oper_table = SymbolTable::<OperId>::new();
    let r = type_parser(&table, &oper_table).easy_parse(type_name_example);
    assert_eq!(
        r,
        Ok((
            Type::Binary(
                OperId(1),
                Rc::new(Type::Unary(TypeId(2))),
                Rc::new(Type::Unary(TypeId(2)))
            ),
            ""
        ))
    );
    dbg!(&table);
    assert_eq!(table.get("Bool"), Some(TypeId(2)));
}

fn parse_unary_type<Input>() -> impl Parser<Input, Output = String>
where
    Input: stream::Stream<Token = char>,
{
    many1(char::alpha_num()).map(move |name_chars: Vec<_>| {
        let name: String = name_chars.into_iter().collect();
        name
    })
}

pub fn type_unary_parser<'a, I>(
    table: &'a SymbolTable<TypeId>,
) -> impl Parser<I, Output = Type> + 'a
where
    I: Stream<Token = char> + 'a,
{
    parse_unary_type::<I>().map(move |name: String| {
        let id = table.assign(name.clone());
        Type::Unary(id)
    })
}

#[test]
fn test_type_unary_parser() {
    use crate::combine::EasyParser;

    let type_name_example = "Bool";
    let table = SymbolTable::<TypeId>::init_with(TypeId(3));
    let r = type_unary_parser(&table).easy_parse(type_name_example);
    assert_eq!(r.unwrap().0, Type::Unary(TypeId(3)));
}

fn type_binary_parser<'a, Input>(
    table: &'a SymbolTable<TypeId>,
    oper_table: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Type> + 'a
where
    Input: stream::Stream<Token = char> + 'a,
{
    many1(char::alpha_num())
        .skip(spaces())
        .and(char::string("*").skip(spaces()))
        .and(many1(char::alpha_num()))
        .map(move |((left, _), right): ((Vec<_>, &str), Vec<_>)| {
            let name_l: String = left.into_iter().collect();
            let name_r: String = right.into_iter().collect();
            // 最新のidを取得する
            let id_l = table.assign(name_l);
            let id_r = table.assign(name_r);
            let oper_id = oper_table.assign("*".to_string());
            Type::Binary(
                oper_id,
                Rc::new(Type::Unary(id_l.clone())),
                Rc::new(Type::Unary(id_r.clone())),
            )
        })
}

#[test]
fn test_type_binary_parser() {
    use crate::combine::EasyParser;
    use std::rc::Rc;

    let type_name_example = "Bool * Bool";
    let table = SymbolTable::<TypeId>::init_with(TypeId(2));
    let oper_table = SymbolTable::<OperId>::new();
    let r = type_binary_parser(&table, &oper_table).easy_parse(type_name_example);
    assert_eq!(
        r,
        Ok((
            Type::Binary(
                OperId(1),
                Rc::new(Type::Unary(TypeId(2))),
                Rc::new(Type::Unary(TypeId(2)))
            ),
            ""
        ))
    );
    dbg!(&table);
    assert_eq!(table.get("Bool"), Some(TypeId(2)));
}
