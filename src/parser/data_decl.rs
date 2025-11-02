use std::{
    collections::HashMap, rc::Rc};

use combine::{
    parser::char::{spaces, string},
    Parser, Stream,
};

use crate::{
    context::Context, context_table::CtxtTable, equation::Equation, id::{OperId}, 
    parser::{DIRECTIVE_SIGN, term::terminner::oper::terminner_parser}, symbol_table::SymbolTable,
};

pub fn data_decl_parser<'a, Input>(
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let left_parser = terminner_parser(ctxts, opers);
    let right_parser = terminner_parser(ctxts, opers);

    string(DIRECTIVE_SIGN)
        .and(string("data"))
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

#[cfg(test)]
mod tests {
    use crate::{context_table::CtxtTable, id::{OperId}, parser::data_decl::data_decl_parser, symbol_table::SymbolTable};
    use combine::EasyParser;
    
    #[test]
    fn test_data_decl_parser() {
        let input = "#data wrk!e1; = d3;";

        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("".to_string());

        let opers = SymbolTable::<OperId>::new();
        opers.assign("wrk".to_string());
        opers.assign("e1".to_string());
        opers.assign("d3".to_string());

        let result = data_decl_parser(&ctxts, &opers).easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());
    }
}
