use combine::{attempt, parser::char::spaces, sep_end_by, Parser, Stream};

use crate::{
    context_table::CtxtTable, equation::Equation, id::{OperId, TypeId}, instance::Instance, oper::Oper, 
    parser::{
        data_decl::data_decl_parser, elem_decl::elem_decl_parser, schema_decl::schema_decl_parser,
    },
    schema::Schema, symbol_table::SymbolTable, 
};

pub fn instance_parser<'a, Input>(
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
        Elem(Vec<Oper>),
        Data(Equation),
    }

    let elem_parser = elem_decl_parser(types, opers);
    let data_parser = data_decl_parser(ctxts, opers);

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
                    // dbg!(&elems);
                    instance.elems.extend(elems);
                }
                Decl::Data(eq) => instance.data.push(eq),
            }
        }
        // dbg!(&instance.elems);
        instance
    })
}

#[test]
fn test_parse_instance() {
    use combine::EasyParser;

    let f = "instance/i.instance";
    let input = std::fs::read_to_string(f).expect("Failed to read");

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();

    let result = instance_parser::<combine::easy::Stream<&str>>(&types, &opers, &ctxts)
        .easy_parse(input.as_ref());
    dbg!(&result);
    assert!(result.is_ok())
}
