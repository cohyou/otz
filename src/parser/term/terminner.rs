mod r#const;
pub mod oper;
mod var;

use crate::id::{OperId, VarId};
use crate::parser::term::terminner::oper::terminner_oper_parser;
use crate::parser::term::terminner::var::terminner_var_parser;
use crate::symbol_table::SymbolTable;
use crate::term::TermInner;
use combine::attempt;
use combine::stream::Stream;
use combine::Parser;

pub fn terminner_parser_<'a, Input>(
    vars: &'a SymbolTable<VarId>,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    attempt(terminner_oper_parser(vars, opers)).or(terminner_var_parser(vars))
}

#[test]
fn test_terminner_parser1() {
    use crate::combine::EasyParser;
    let opers = SymbolTable::<OperId>::new();
    opers.insert("f".to_string(), OperId(0));
    let vars = SymbolTable::<VarId>::new();
    vars.insert("a".to_string(), VarId(0));

    let r = terminner_oper_parser(&vars, &opers).easy_parse("f![a]");
    dbg!(&opers);
    assert_eq!(
        r,
        Ok((
            TermInner::Fun(OperId(0), vec![TermInner::Var(VarId(0)).into()]),
            ""
        ))
    );
}

#[test]
fn test_terminner_parser2() {
    use crate::combine::EasyParser;
    let opers = SymbolTable::<OperId>::new();
    opers.insert("f".to_string(), OperId(0));
    let vars = SymbolTable::<VarId>::new();
    vars.insert("a".to_string(), VarId(0));

    let r = terminner_oper_parser(&vars, &opers).easy_parse("f![f![a]]");
    dbg!(&opers);
    assert_eq!(
        r,
        Ok((
            TermInner::Fun(
                OperId(0),
                vec![TermInner::Fun(OperId(0), vec![TermInner::Var(VarId(0)).into()]).into()]
            ),
            ""
        ))
    );
}

// fn terminner_parser<'b, 'a, Input>(
//     generator: AsyncIncrement<OperId>,
//     _types: &'b TypeSymbolTable,
//     _opers: &'b OperSymbolTable,
// ) -> impl Parser<Input, Output = TermInner> + 'b
// where
//     Input: stream::Stream<Token = char> + 'b,
// {
//     string(DIRECTIVE_SIGN)
//         .and(string("term"))
//         .and(spaces())
//         .with(many1(alpha_num()))
//         .map(move |c: Vec<_>| {
//             let _name: String = c.into_iter().collect();
//             // 最新のidを取得する
//             let id = generator.pull();
//             // symbol_table.insert(name, id.clone());
//             TermInner::Fun(id.clone(), vec![]) // Placeholder for function arguments
//         })
//     // This parser would be implemented similarly to the oper_parser,
//     // but for terms instead of operations.
// }

// #[test]
// fn test_terminner_parser() {
//     use crate::combine::EasyParser;
//     let term_example = "a";
//     let generator = OperId::init_with(OperId(3));
//     let mut symbol_table = OperSymbolTable::new();
//     let types = TypeSymbolTable::new();
//     let r = terminner_parser(generator, &mut symbol_table, &types).easy_parse(term_example);
//     dbg!(&symbol_table);
//     assert_eq!(symbol_table.get("apply"), Some(&OperId(3)));
// }
