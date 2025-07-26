use std::fs::read_to_string;

use combine::stream::Stream;
use combine::{EasyParser, Parser};

use combine::parser::char::{alpha_num, spaces, string};
use combine::{between, many1};

use crate::context_table::CtxtTable;
use crate::id::{OperId, TypeId};
use crate::parser::theory::theory_parser;
use crate::symbol_table::SymbolTable;
use crate::theory::Theory;

use crate::parser::DIRECTIVE_SIGN;

pub fn theory_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Theory> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string::<Input>(DIRECTIVE_SIGN)
        .and(string("theory"))
        .and(spaces())
        .with(between(
            string("\""),
            string("\""),
            many1::<Vec<_>, Input, _>(alpha_num()),
        ))
        .map(|chars: Vec<_>| {
            let name = chars.into_iter().collect::<String>();
            let path = format!("theory/{}.theory", name);
            let src =
                read_to_string(&path).expect(&format!("Failed to read theory file: {}", path));
            let x = theory_parser::<combine::easy::Stream<&str>>(types, opers, ctxts)
                .easy_parse(src.as_ref())
                .expect(&format!("Failed to parse theory from file: {}", path))
                .0;
            x
        })
}

#[test]
fn test_theory_decl_parser() {
    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();

    let example = "#theory \"test\"";

    let r = theory_decl_parser(&types, &opers, &ctxts).easy_parse(example);
    dbg!(&r);
    assert!(r.is_ok());
}
