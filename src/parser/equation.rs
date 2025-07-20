use crate::equation::Equation;
use crate::id::{OperId, TypeId, VarId};
use crate::parser::context::context_parser;
use crate::parser::term::terminner::oper::terminner_parser;
use crate::symbol_table::SymbolTable;

use combine::parser::char::spaces;
use combine::parser::char::string;
use combine::stream::Stream;
use combine::Parser;

pub fn equation_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    vars: &'a SymbolTable<VarId>,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let context_parser = context_parser::<Input>(vars, types);
    let left_parser = terminner_parser(vars, opers);
    let right_parser = terminner_parser(vars, opers);

    (
        context_parser
            .skip(spaces())
            .skip(string("|"))
            .skip(spaces()),
        left_parser.skip(spaces()).skip(string("=").skip(spaces())),
        right_parser,
    )
        .map(|(context, left, right)| Equation {
            context: context.clone(),
            left,
            right,
        })
}

#[test]
fn test_terminner_equation_parser() {
    use crate::combine::EasyParser;
    use crate::context::Ctxt;
    use crate::r#type::Type;
    use crate::term::TermInner;

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    opers.insert("f".to_string(), OperId(0));
    let vars = SymbolTable::<VarId>::new();
    vars.insert("a".to_string(), VarId(0));
    let r = equation_parser(&types, &vars, &opers).easy_parse("a: Bool | f![f![a]] = a");
    dbg!(&opers);
    assert_eq!(
        r,
        Ok((
            Equation {
                context: Ctxt(vec![Type::Unary(TypeId(1))].into()),
                left: TermInner::Fun(
                    OperId(0),
                    vec![TermInner::Fun(OperId(0), vec![TermInner::Var(VarId(0)).into()]).into()]
                ),
                right: TermInner::Var(VarId(0)),
            },
            ""
        ))
    );
}
