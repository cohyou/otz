use std::rc::Rc;

use combine::{parser::char::{spaces, string}, Parser, Stream};

use crate::{context_table::CtxtTable, id::{OperId, TypeId}, parser::{context::context_parser, term::terminner::oper::terminner_parser}, symbol_table::SymbolTable, term::Term};

pub mod terminner;

pub fn term_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Term> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let context_parser = context_parser::<Input>(ctxts, types);
    let inner_parser = terminner_parser(ctxts, opers);

    context_parser
        .skip(spaces())
        .skip(string("|"))
        .skip(spaces())
        .and(inner_parser)
        .map(|(context, inner)| {
            ctxts.complete();
            Term {
                context,
                inner: Rc::new(inner),
            }
        })
}

#[cfg(test)]
mod test {
    use crate::{context_table::CtxtTable, id::{OperId, TypeId}, parser::{term::term_parser}, symbol_table::SymbolTable};

    #[test]
    fn test_term_parser() {
        use crate::combine::EasyParser;

        let example = "a: Bool | f![f![a]]";

        let types = SymbolTable::<TypeId>::new();
        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("a".to_string());
        let term = term_parser(&types, &ctxts, &opers).easy_parse(example);
        dbg!(&types);
        dbg!(&opers);
        dbg!(&term);
    }
}