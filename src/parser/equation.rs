use combine::parser::char::{spaces, string};
use combine::stream::Stream;
use combine::Parser;

use crate::equation::Equation;
use crate::id::{OperId, TypeId};
use crate::parser::context::context_parser;
use crate::parser::term::terminner::oper::terminner_parser;
use crate::symbol_table::SymbolTable;
use crate::context_table::CtxtTable;

pub fn equation_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let context_parser = context_parser::<Input>(ctxts, types);
    let left_parser = terminner_parser(ctxts, opers);
    let right_parser = terminner_parser(ctxts, opers);

    
        context_parser
            .skip(spaces())
            .skip(string("|"))
            .skip(spaces())
        .and(left_parser.skip(spaces()).skip(string("=").skip(spaces())))
        .and(right_parser)
        .map(|((context, left), right)| {
            ctxts.complete();
            Equation {
                context,
                left,
                right,
            }
        })
    // ;
    // combine::value(Equation {
    //     context: crate::context::Ctxt(std::collections::HashMap::default()),
    //     left: crate::term::TermInner::Var(VarId(0)),
    //     right: crate::term::TermInner::Var(VarId(0)),
    // })
}

#[test]
fn test_terminner_equation_parser() {
    use crate::combine::EasyParser;
    use crate::context::Ctxt;
    use crate::term::TermInner;
    use crate::id::VarId;
    use crate::id::CtxtId;
    use std::collections::HashMap;
    
    let example = "a: Bool | f![f![a]] = a";

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    opers.assign("f".to_string());
    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("a".to_string());
    let r = equation_parser(&types, &ctxts, &opers).easy_parse(example);
    dbg!(&types);
    dbg!(&opers);

    let mut vars = HashMap::new();
    vars.insert(VarId(0), crate::r#type::Type::Unary(TypeId(1))); // Mocking a type for testing
    let mut context = Ctxt::default();
    context.0.insert(CtxtId(1), vars);
    assert_eq!(
        r,
        Ok((
            Equation {
                context: context,
                left: TermInner::Fun(
                    OperId(1),
                    vec![TermInner::Fun(OperId(1), vec![TermInner::Var(VarId(0)).into()]).into()]
                ),
                right: TermInner::Var(VarId(0)),
            },
            ""
        ))
    );
}
