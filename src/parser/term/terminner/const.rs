use combine::{
    many1,
    parser::char::{alpha_num, string},
    Parser, Stream,
};

use crate::{id::OperId, symbol_table::SymbolTable, term::TermInner};

pub fn terminner_const_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num()).skip(string("!")).map(|c: Vec<_>| {
        let name: String = c.into_iter().collect();
        let oper_id = opers
            .get(name.as_ref())
            .expect(format!("Oper '{}' not found in symbol table", name).as_ref());
        TermInner::Fun(oper_id, vec![])
    })
}

#[test]
fn test_terminner_const_parser() {
    use crate::combine::EasyParser;

    let opers = SymbolTable::<OperId>::new();
    opers.assign("f".to_string());

    let input = "f!";
    let result = terminner_const_parser(&opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
