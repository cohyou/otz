use std::rc::Rc;

use combine::{
    parser::char::{spaces, string},
    Parser, Stream,
};

use crate::{
    context_table::CtxtTable,
    id::{OperId, TypeId},
    parser::{context::context_parser, term::terminner::oper::terminner_parser},
    symbol_table::SymbolTable,
    term::Term,
};

pub mod terminner;

pub fn term_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Term> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let context_parser = context_parser::<Input>(types, ctxts);
    let inner_parser = terminner_parser(ctxts, opers);

    context_parser
        .skip(spaces())
        .skip(string("|"))
        .skip(spaces())
        .and(inner_parser)
        .map(|(context, inner)| {
            let mut names = ctxts.current_var_table();
            let oper_names = opers.current_table();
            names.extend(oper_names);
            Term {
                context: context.into(),
                names: names.into(),
                inner: Rc::new(inner),
            }
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::term::term_parser,
        util::{opers, types, vars},
    };

    #[test]
    fn test_term_parser() {
        use crate::combine::EasyParser;

        let example = "a: Bool | f!a";
        let types = types(vec!["Bool"]);
        let opers = opers(vec!["f"]);
        let ctxts = vars(vec![]);
        let term = term_parser(&types, &opers, &ctxts)
            .easy_parse(example)
            .unwrap()
            .0;
        println!("{}", &term);
    }
}
