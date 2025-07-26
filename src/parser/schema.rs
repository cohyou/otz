use combine::parser::char::spaces;
use combine::stream::Stream;
use combine::Parser;
use combine::{attempt, sep_end_by};

use crate::schema::Schema;

use crate::context_table::CtxtTable;
use crate::id::{OperId, TypeId};
use crate::symbol_table::SymbolTable;

use crate::equation::Equation;
use crate::oper::Oper;
use crate::r#type::Type;
use crate::theory::Theory;

use crate::parser::attr_decl::attr_decl_parser;
use crate::parser::eq_decl::equation_decl_parser;
use crate::parser::fkey_decl::fkey_decl_parser;
use crate::parser::theory_decl::theory_decl_parser;
use crate::parser::type_decl::type_decl_parser;

pub fn schema_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Schema> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    #[derive(Clone)]
    enum Decl {
        Theory(Theory),
        Entity(Type),
        Fkey(Oper),
        Attr(Oper),
        Equation(Equation),
    }

    let theory_decl_parser = theory_decl_parser::<Input>(types, opers, ctxts);
    let entity_parser = type_decl_parser::<Input>(types);
    let fkey_parser = fkey_decl_parser::<Input>(opers, types);
    let attr_parser = attr_decl_parser::<Input>(opers, types);
    let equation_parser = equation_decl_parser::<Input>(types, ctxts, opers);

    let decl_parsers = attempt(theory_decl_parser.map(Decl::Theory))
        .or(attempt(entity_parser.map(Decl::Entity)))
        .or(attempt(fkey_parser.map(Decl::Fkey)))
        .or(attempt(attr_parser.map(Decl::Attr)))
        .or(equation_parser.map(Decl::Equation));
    let separator = spaces::<Input>();

    sep_end_by(decl_parsers, separator).map(|decls: Vec<Decl>| {
        let mut schema = Schema::default();

        for decl in decls {
            match decl {
                Decl::Theory(th) => schema.theory = th,
                Decl::Entity(ty) => schema.entities.push(ty),
                Decl::Fkey(op) => schema.fkeys.push(op),
                Decl::Attr(op) => schema.attrs.push(op),
                Decl::Equation(eq) => schema.constraints.push(eq),
            }
        }
        schema
    })
}

#[test]
fn test_schema_parser() {
    use crate::combine::EasyParser;

    let f = "schema/s.schema";
    let schema_example = std::fs::read_to_string(f).expect("Failed to read");

    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();
    let result = schema_parser::<combine::easy::Stream<&str>>(&types, &opers, &ctxts)
        .easy_parse(schema_example.as_ref());

    dbg!(&types);
    dbg!(&opers);
    dbg!(&ctxts);
    dbg!(&result);
    match result {
        Ok((_, _)) => {}
        Err(err) => panic!("Failed to parse schema: {}", err),
    }
}
