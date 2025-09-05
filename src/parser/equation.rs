use std::rc::Rc;

use combine::{
    parser::char::{spaces, string},
    Parser, Stream,
};

use crate::{
    context_table::CtxtTable,
    equation::Equation,
    id::{OperId, TypeId},
    parser::{context::context_parser, term::terminner::oper::terminner_parser},
    symbol_table::SymbolTable,
};

pub fn equation_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
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
            // ctxts.complete();
            let mut names = ctxts.current_var_table();
            let oper_names = opers.current_table();
            names.extend(oper_names);
            Equation {
                context,
                names,
                left: Rc::new(left),
                right: Rc::new(right),
            }
        })
}

#[cfg(test)]
mod test {
    use combine::EasyParser;

    use crate::{
        context_table::CtxtTable,
        parser::equation::equation_parser,
        util::{opers, types},
    };

    #[test]
    fn test_equation_parser() {
        let types = types(vec![]);
        let opers = opers(vec!["f"]);
        let ctxts = CtxtTable::new();

        let input = "a: Bool | f![f![a]] = a";
        let result = equation_parser(&types, &opers, &ctxts).easy_parse(input);
        println!("{}", &result.unwrap().0);
    }
}
