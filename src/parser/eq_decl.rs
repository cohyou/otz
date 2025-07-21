use combine::Parser;
use combine::stream::Stream;
use combine::parser::char::{spaces, string};

use crate::equation::Equation;
use crate::id::{OperId, TypeId};

use crate::parser::equation::equation_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::symbol_table::SymbolTable;
use crate::context_table::CtxtTable;


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

#[ignore]
#[test]
fn test_type_decl_parser() {
    use crate::combine::EasyParser;

    let type_name_example = "#sort Bool";

    let opers = SymbolTable::<OperId>::init_with(OperId(2));
    let types = SymbolTable::<TypeId>::init_with(TypeId(2));
    let ctxts = CtxtTable::new();
    let _r = equation_decl_parser(&types, &ctxts, &opers).easy_parse(type_name_example);
    dbg!(&opers);
    assert_eq!(types.get("Bool"), Some(TypeId(2)));
}
