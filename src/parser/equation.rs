use std::rc::Rc;

use combine::parser::char::{spaces, string};
use combine::stream::Stream;
use combine::Parser;

use crate::context_table::CtxtTable;
use crate::equation::Equation;
use crate::id::{OperId, TypeId};
use crate::parser::context::context_parser;
use crate::parser::term::terminner::oper::terminner_parser;
use crate::symbol_table::SymbolTable;

pub fn equation_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let context_parser = context_parser::<Input>(types, ctxts);
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
                left: Rc::new(left),
                right: Rc::new(right),
            }
        })
}

#[test]
fn test_terminner_equation_parser() {
    use crate::combine::EasyParser;
    use crate::context::Context;
    use crate::id::VarId;
    use crate::term::TermInner;
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
    let context = Context(vars);
    assert_eq!(
        r,
        Ok((
            Equation {
                context: context,
                left: Rc::new(TermInner::Fun(
                    OperId(1),
                    vec![TermInner::Fun(OperId(1), vec![TermInner::Var(VarId(0)).into()]).into()]
                )),
                right: Rc::new(TermInner::Var(VarId(0))),
            },
            ""
        ))
    );
}
