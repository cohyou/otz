use crate::equation::Equation;
use crate::id::{OperId, TypeId, VarId};
use crate::oper::Oper;
use crate::parser::equation::equation_parser;
use crate::parser::DIRECTIVE_SIGN;
use crate::symbol_table::SymbolTable;
use combine::parser::char;
use combine::parser::char::spaces;
use combine::stream::Stream;
use combine::Parser;

/// 型名
pub fn equation_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    vars: &'a SymbolTable<VarId>,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    char::string(DIRECTIVE_SIGN)
        .and(char::string("rule"))
        .and(spaces())
        .with(equation_parser(types, vars, opers))
}

#[ignore]
#[test]
fn test_type_decl_parser() {
    use crate::combine::EasyParser;

    let type_name_example = "#sort Bool";

    let opers = SymbolTable::<OperId>::init_with(OperId(2));
    let types = SymbolTable::<TypeId>::init_with(TypeId(2));
    let vars = SymbolTable::<VarId>::init_with(VarId(1));
    let _r = equation_decl_parser(&types, &vars, &opers).easy_parse(type_name_example);
    dbg!(&opers);
    assert_eq!(types.get("Bool"), Some(TypeId(2)));
}
