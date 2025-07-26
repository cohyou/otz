use combine::{
    many1,
    parser::char::{alpha_num, string},
    Parser, Stream,
};

use crate::{
    context_table::CtxtTable, id::OperId, parser::term::terminner::oper::terminner_parser,
    symbol_table::SymbolTable, term::TermInner,
};

pub fn terminner_oper_unary_parser<'a, Input>(
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num())
        .skip(string("!"))
        .and(terminner_parser(ctxts, opers))
        .map(|(c, v): (Vec<_>, _)| {
            let name: String = c.into_iter().collect();
            let oper_id = opers
                .get(name.as_ref())
                .expect(format!("Oper '{}' not found in symbol table", name).as_ref());
            TermInner::Fun(oper_id, vec![v.into()])
        })
}

#[test]
fn test_terminner_oper_unary_parser() {
    use crate::combine::EasyParser;

    let opers = SymbolTable::<OperId>::new();
    opers.assign("f".to_string());
    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("a".to_string());

    let input = "f!a";
    let result = terminner_oper_unary_parser(&ctxts, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
