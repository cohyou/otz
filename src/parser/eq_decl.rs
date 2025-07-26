use combine::parser::char::{spaces, string};
use combine::stream::Stream;
use combine::Parser;

use crate::equation::Equation;
use crate::id::{OperId, TypeId};

use crate::context_table::CtxtTable;
use crate::parser::equation::equation_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::symbol_table::SymbolTable;

pub fn equation_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string(DIRECTIVE_SIGN)
        .and(string("rule"))
        .and(spaces())
        .with(equation_parser(types, ctxts, opers))
}

#[test]
fn test_eq_decl_parser() {
    use crate::combine::EasyParser;

    let type_name_example = "#rule a: Bool | not![a] = a";

    let types = SymbolTable::<TypeId>::new();
    types.assign("Bool".to_string());
    let opers = SymbolTable::<OperId>::new();
    opers.assign("not".to_string());
    let ctxts = CtxtTable::new();
    let result = equation_decl_parser(&types, &ctxts, &opers).easy_parse(type_name_example);
    dbg!(&result);
    assert!(result.is_ok());
}
