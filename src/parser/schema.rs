use combine::Parser;
use combine::stream::Stream;
use combine::{attempt, sep_end_by, any, many};
use combine::parser::char::{spaces, string, newline};
use combine::parser::repeat::skip_until;

use crate::schema::Schema;

use crate::symbol_table::SymbolTable;
use crate::id::{OperId, TypeId, VarId};
use crate::parser::theory::theory_parser;

use crate::r#type::Type;
use crate::oper::Oper;
use crate::equation::Equation;

use crate::parser::eq_decl::equation_decl_parser;
use crate::parser::fkey_decl::fkey_decl_parser;
use crate::parser::attr_decl::attr_decl_parser;
use crate::parser::type_decl::type_decl_parser;


fn schema_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    vars: &'a SymbolTable<VarId>,
)
 -> impl Parser<Input, Output = Schema> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    use crate::combine::EasyParser;
    use combine::easy::Stream;

    #[derive(Clone)]
    enum Decl {
        Schema,
        Type(Type),
        Fkey(Oper),
        Attr(Oper),
        Equation(Equation),
    }

    let schema_parser = string::<Input>("#theory \"test\"");
    let type_parser = type_decl_parser::<Input>(&types);
    let fkey_parser = fkey_decl_parser::<Input>(&opers, &types);
    let attr_parser = attr_decl_parser::<Input>(&opers, &types);
    let equation_parser = equation_decl_parser::<Input>(&types, &vars, &opers);

    let decl_parsers = attempt(schema_parser.map(|_| Decl::Schema))
        .or(attempt(type_parser.map(Decl::Type)))
        .or(attempt(fkey_parser.map(Decl::Fkey)))
        .or(attempt(attr_parser.map(Decl::Attr)))
        .or(equation_parser.map(Decl::Equation));
    let separator = spaces::<Input>();

    sep_end_by(decl_parsers, separator).map(|decls: Vec<Decl>| {
        let mut schema = Schema::default();

        let f = "theory/test.theory";
        let theory_example = std::fs::read_to_string(f).expect("Failed to read");
        let types = SymbolTable::<TypeId>::new();
        let opers = SymbolTable::<OperId>::new();
        let vars = SymbolTable::<VarId>::new();
        let r = theory_parser::<Stream<&str>>(&types, &opers, &vars).easy_parse(theory_example.as_ref());
        match r {
            Ok((theory, _)) => {
                schema.theory = theory;
            }
            Err(err) => panic!("Failed to parse theory: {}", err),
        }

        for decl in decls {
            match decl {
                Decl::Schema => continue, // Schema declaration is handled separately
                Decl::Type(ty) => schema.theory.types.push(ty),
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
    let vars = SymbolTable::<VarId>::new();
    let result = schema_parser::<combine::easy::Stream<&str>>(&types, &opers, &vars).easy_parse(schema_example.as_ref());
    dbg!(&result);
    match result {
        Ok((schema, _)) => {
            // assert_eq!(schema.theory.name, "ExampleSchema");
            assert!(schema.entities.is_empty());
            assert!(schema.fkeys.is_empty());
            assert!(schema.attrs.is_empty());
            assert!(schema.constraints.is_empty());
        }
        Err(err) => panic!("Failed to parse schema: {}", err),
    }    
}
