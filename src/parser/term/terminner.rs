use combine::{attempt, Parser, Stream};

use crate::{
    context_table::CtxtTable,
    id::OperId,
    parser::term::terminner::{
        oper::terminner_oper_parser, oper_unary::terminner_oper_unary_parser,
        r#const::terminner_const_parser, var::terminner_var_parser,
    },
    symbol_table::SymbolTable,
    term::TermInner,
};

mod r#const;
pub mod oper;
mod oper_unary;
// mod oper_post;
mod var;

pub fn terminner_parser_<'a, Input>(
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    attempt(terminner_oper_parser(ctxts, opers))
        .or(attempt(terminner_oper_unary_parser(ctxts, opers)))
        .or(attempt(terminner_const_parser(opers)))
        .or(attempt(terminner_var_parser(ctxts)))
    // let parser = terminner_oper_post_parser(ctxts, opers);
}

// fn terminner_parser_inner<'a, Input>(
//     ctxts: &'a CtxtTable,
//     opers: &'a SymbolTable<OperId>,
// ) -> impl Parser<Input, Output = TermInner> + 'a
// where
//     Input: Stream<Token = char> + 'a,
// {

// }

#[test]
fn test_terminner_parser1() {
    use crate::combine::EasyParser;

    let opers = SymbolTable::<OperId>::new();
    opers.assign("f".to_string());
    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("a".to_string());

    let input = "f![a]";
    let result = terminner_oper_parser(&ctxts, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}

#[test]
fn test_terminner_parser2() {
    use combine::EasyParser;

    let opers = SymbolTable::<OperId>::new();
    opers.assign("f".to_string());
    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("a".to_string());

    let input = "f![f![a]]";
    let result = terminner_oper_parser(&ctxts, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_parse_terminner_oper_unary_nested() {
    use combine::EasyParser;

    let input = "name!name!e";

    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("e".to_string());
    let opers = SymbolTable::<OperId>::new();
    opers.assign("name".to_string());
    let result = terminner_oper_unary_parser(&ctxts, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
