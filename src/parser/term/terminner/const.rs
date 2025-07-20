use crate::id::OperId;
use crate::symbol_table::SymbolTable;
use crate::term::TermInner;
use combine::many1;
use combine::parser::char::{alpha_num, string};
use combine::stream::Stream;
use combine::Parser;

fn terminner_const_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num()).skip(string("![]")).map(|c: Vec<_>| {
        let name: String = c.into_iter().collect();
        let oper_id = opers
            .get(name.as_ref())
            .expect(format!("Oper '{}' not found in symbol table", name).as_ref());
        TermInner::Fun(oper_id, vec![])
    })
}

#[test]
fn test_terminner_var_parser() {
    use crate::combine::EasyParser;
    let opers = SymbolTable::<OperId>::new();
    opers.insert("f".to_string(), OperId(0));
    let r = terminner_const_parser(&opers).easy_parse("f![]");
    dbg!(&opers);
    assert_eq!(r, Ok((TermInner::Fun(OperId(0), vec![]), "")));
}
