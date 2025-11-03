use std::{collections::HashMap, rc::Rc};

use combine::{Parser, Stream, parser::char::{spaces, string}};

use crate::{context::Context, context_table::CtxtTable, equation::Equation, id::OperId, parser::{DIRECTIVE_SIGN, term::terminner::oper::terminner_parser}, symbol_table::SymbolTable};

pub fn where_decl_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let left_parser = terminner_parser(ctxts, opers);
    let right_parser = terminner_parser(ctxts, opers);

    string(DIRECTIVE_SIGN)
        .and(string("where"))
        .and(spaces())
        .with(left_parser.skip(spaces()).skip(string("=").skip(spaces())))
        .and(right_parser)
        .map(|(left, right)| -> Equation {
            let context = Context(HashMap::new());
            let mut names = ctxts.current_var_table();
            let oper_names = opers.current_table();
            names.extend(oper_names);
            Equation {
                context: context.into(),
                names: names.into(),
                left: Rc::new(left),
                right: Rc::new(right),
            }
        })
}

