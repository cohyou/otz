use combine::{Parser, Stream, many1, parser::char::{alpha_num, spaces, string}};

use crate::{context_table::CtxtTable, id::{OperId}, parser::{DIRECTIVE_SIGN, term::terminner::oper::terminner_parser}, symbol_table::SymbolTable, term::TermInner};

pub fn return_decl_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = (OperId, TermInner)> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let left_parser = return_oper_parser(opers);
    let right_parser = terminner_parser(ctxts, opers);

    string(DIRECTIVE_SIGN)
        .and(string("return"))
        .and(spaces())
        .with(left_parser.skip(spaces()).skip(string(":=").skip(spaces())))
        .and(right_parser)
        .map(|(operid, right)| {
            (operid, right)
        })
}

fn return_oper_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = OperId> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num())
        .map(move |name: String| {
            opers.assign(name)
        })
}

#[cfg(test)]
mod tests {
    use crate::{context_table::CtxtTable, id::{OperId}, parser::query::return_decl::return_decl_parser, symbol_table::SymbolTable};
    use combine::EasyParser;
     
    #[test]
    fn test_return_decl_parser() {
        let input = "#return name := last!e";
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("e".to_string());
        let opers = SymbolTable::<OperId>::new();
        opers.insert("last".to_string(), OperId(2)); // Mocking an oper for testing
        let result = return_decl_parser(&opers, &ctxts).easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());
    }
}   