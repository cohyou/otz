use std::{cell::RefCell, collections::HashMap};

use combine::{attempt, parser::char::spaces, sep_end_by, Parser, Stream};

use crate::{
    context::Context, context_table::CtxtTable, equation::Equation, id::{OperId, TypeId, VarId}, instance::Instance, parser::{
        data_decl::data_decl_parser, elem_decl::elem_decl_parser, schema_decl::schema_decl_parser,
    }, schema::Schema, symbol_table::SymbolTable, r#type::Type
};

pub fn instance_parser<'a, Input>(
    elems: &'a RefCell<HashMap<VarId, Type>>,
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Instance> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    #[derive(Clone)]
    enum Decl {
        Schema(Schema),
        Elem(HashMap<VarId, Type>),
        Data(Equation),
    }

    let elem_parser = elem_decl_parser(elems, ctxts, types);
    let data_parser = data_decl_parser(elems, ctxts, opers);

    // schema: Schema,
    let decl_parsers = attempt(schema_decl_parser(types, opers, ctxts).map(Decl::Schema))
        .or(attempt(elem_parser.map(Decl::Elem)))
        .or(data_parser.map(Decl::Data));

    sep_end_by(decl_parsers, spaces()).map(|decls: Vec<Decl>| {
        let mut instance = Instance::default();

        for decl in decls {
            match decl {
                Decl::Schema(sch) => instance.schema = sch,
                Decl::Elem(elems) => {
                    dbg!(&elems);
                    instance.elems = Context(elems);
                    // instance.elems.extend_to_default(elem)
                }
                Decl::Data(eq) => instance.data.push(eq),
            }
        }
        dbg!(&instance.elems);
        instance
    })
}

#[test]
fn test_parse_instance() {
    use combine::EasyParser;

    let f = "instance/i.instance";
    let input = std::fs::read_to_string(f).expect("Failed to read");

    let elems = RefCell::new(HashMap::new());
    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();

    let result = instance_parser::<combine::easy::Stream<&str>>(&elems, &types, &opers, &ctxts)
        .easy_parse(input.as_ref());
    dbg!(&result);
    assert!(result.is_ok())
}
