use std::rc::Rc;

use combine::{
    parser::char::{spaces, string},
    Parser, Stream,
};

use crate::{
    context_table::CtxtTable,
    id::{OperId, TypeId},
    parser::{context::context_parser, term::terminner::oper::terminner_parser},
    rule::Rule,
    symbol_table::SymbolTable,
};

pub fn rule_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Rule> + 'a
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
        .and(left_parser.skip(spaces()).skip(string("->").skip(spaces())))
        .and(right_parser)
        .map(|((context, before), after)| {
            // ctxts.complete();
            Rule::new(context.clone(), Rc::new(before), Rc::new(after))
        })
}
