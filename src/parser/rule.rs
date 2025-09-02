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
    let context_parser = context_parser::<Input>(types, ctxts);
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
            let mut names = ctxts.current_var_table();
            let oper_names = opers.current_table();
            names.extend(oper_names);
            Rule::new(context.clone(), names, Rc::new(before), Rc::new(after))
        })
}
