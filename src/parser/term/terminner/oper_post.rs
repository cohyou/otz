
use combine::{
    value, Parser, Stream,
};

use crate::{
    context_table::CtxtTable,
    id::{OperId, VarId},
    symbol_table::SymbolTable,
    term::TermInner,
};

pub fn terminner_oper_post_parser<'a, Input>(
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    // let token_parser_left = terminner_parser(ctxts, opers);
    // let token_parser_right = many1(alpha_num());
    // token_parser_left
    //     .skip(string("."))
    //     .and(token_parser_right)

    // let parser = terminner_parser(ctxts, opers);
    // let op = string(".")
    // .map(|(left, right): (TermInner, Vec<_>)| {
    //     let oper_name: String = right.into_iter().collect();
    //     let oper_id = opers
    //         .get(oper_name.as_ref())
    //         .expect(format!("Oper '{}' not found in symbol table", oper_name).as_ref());
    //     TermInner::Fun(oper_id, vec![left.into()])
    // });

    // chainl1(parser, op)

    value(TermInner::Var(VarId(0)))
}

#[test]
fn test_parse_terminner_oper_post() {
    use combine::EasyParser;

    let input = "e.name";

    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("e".to_string());
    let opers = SymbolTable::<OperId>::new();
    opers.assign("name".to_string());
    let result = terminner_oper_post_parser(&ctxts, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}

// #[test]
// fn test_chainl1() {
//     // use combine::EasyParser;

//     let input = "a!b!c";
//     let parser = many1(alpha_num());
//     let op = string("!").map(|l,r|{});
//     let result = chainl1(parser, op).parse(input);
//     dbg!(&result);
//     assert!(result.is_ok());
// }
