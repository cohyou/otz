use std::fs::read_to_string;

use combine::{EasyParser, Parser, Stream, many1, parser::char::{alpha_num, spaces, string}};

use crate::{context_table::CtxtTable, id::{OperId, TypeId}, instance::Instance, parser::{DIRECTIVE_SIGN, instance::instance_parser}, symbol_table::SymbolTable};

pub fn instance_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Instance> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string::<Input>(DIRECTIVE_SIGN)
        .and(string("instance"))
        .and(spaces())
        .with(many1::<Vec<_>, Input, _>(alpha_num()))
        .map(|chars: Vec<_>| {
            let name = chars.into_iter().collect::<String>();
            let path = format!("example/instance/{}.instance", name);
            let src =
                read_to_string(&path).expect(&format!("Failed to read theory file: {}", path));
            let instance = instance_parser::<combine::easy::Stream<&str>>(types, opers, ctxts)
                .easy_parse(src.as_ref())
                .expect(&format!("Failed to parse theory from file: {}", path))
                .0;
            instance
        })
}