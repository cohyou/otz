use std::fs::read_to_string;

use combine::{
    many1,
    parser::char::{alpha_num, spaces, string},
    EasyParser, Parser, Stream,
};

use crate::{
    context_table::CtxtTable,
    id::{OperId, TypeId},
    parser::{schema::schema_parser, DIRECTIVE_SIGN},
    schema::Schema,
    symbol_table::SymbolTable,
};

pub fn schema_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Schema> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string::<Input>(DIRECTIVE_SIGN)
        .and(string("schema"))
        .and(spaces())
        .with(many1::<Vec<_>, Input, _>(alpha_num()))
        .map(|chars: Vec<_>| {
            let name = chars.into_iter().collect::<String>();
            let path = format!("schema/{}.schema", name);
            let src =
                read_to_string(&path).expect(&format!("Failed to read theory file: {}", path));
            let schema = schema_parser::<combine::easy::Stream<&str>>(types, opers, ctxts)
                .easy_parse(src.as_ref())
                .expect(&format!("Failed to parse theory from file: {}", path))
                .0;
            schema
        })
}

#[test]
fn test_parse_schema_decl() {
    use combine::EasyParser;

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();

    let result = schema_decl_parser(&types, &opers, &ctxts).easy_parse("#schema s");
    dbg!(&result);
    assert!(result.is_ok());
}
