use combine::{attempt, Parser, Stream};

use crate::{
    context_table::CtxtTable,
    id::OperId,
    parser::term::terminner::{
        integer::integer_parser, oper::terminner_oper_parser,
        oper_unary::terminner_oper_unary_parser, r#const::terminner_const_parser,
        string::string_parser, var::terminner_var_parser,
    },
    symbol_table::SymbolTable,
    term::TermInner,
};

mod r#const;
pub mod oper;
pub mod oper_unary;
// mod oper_post;
mod integer;
mod string;
mod var;

pub fn terminner_parser_<'a, Input>(
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    attempt(string_parser())
        .or(attempt(integer_parser()))
        .or(attempt(terminner_oper_parser(ctxts, opers)))
        .or(attempt(terminner_oper_unary_parser(ctxts, opers)))
        .or(attempt(terminner_const_parser(opers)))
        .or(terminner_var_parser(ctxts))
}

#[cfg(test)]
mod test {
    use combine::EasyParser;

    use crate::{
        context_table::CtxtTable,
        id::OperId,
        parser::term::terminner::{
            oper::terminner_oper_parser, oper_unary::terminner_oper_unary_parser,
        },
        symbol_table::SymbolTable,
    };

    #[test]
    fn test_terminner_parser1() {
        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("a".to_string());

        let input = "f![a]";
        let result = terminner_oper_parser(&ctxts, &opers).easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_terminner_parser2() {
        use combine::EasyParser;

        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("a".to_string());

        let input = "f![f![a]]";
        let result = terminner_oper_parser(&ctxts, &opers).easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());
    }

    // #[ignore]
    #[test]
    fn test_parse_terminner_oper_unary_nested() {
        use combine::EasyParser;

        let input = "f!f!e";

        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("e".to_string());
        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        let result = terminner_oper_unary_parser(&ctxts, &opers).easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());
    }
}
