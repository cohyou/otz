use combine::parser::char::spaces;
use combine::stream::Stream;
use combine::Parser;
use combine::{attempt, sep_end_by};

use crate::id::{OperId, TypeId};

use crate::equation::Equation;
use crate::oper::Oper;
use crate::r#type::Type;

use crate::context_table::CtxtTable;
use crate::parser::eq_decl::equation_decl_parser;
use crate::parser::oper_decl::oper_decl_parser;
use crate::parser::type_decl::type_decl_parser;
use crate::symbol_table::SymbolTable;
use crate::theory::Theory;

pub fn theory_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Theory> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    #[derive(Clone)]
    enum Decl {
        Type(Type),
        Oper(Oper),
        Equation(Equation),
    }

    let type_parser = type_decl_parser::<Input>(types);
    let oper_parser = oper_decl_parser::<Input>(opers, types);
    let equation_parser = equation_decl_parser::<Input>(types, ctxts, opers);

    let decl_parsers = attempt(type_parser.map(Decl::Type))
        .or(attempt(oper_parser.map(Decl::Oper)))
        .or(equation_parser.map(Decl::Equation));
    let separator = spaces::<Input>();

    sep_end_by(decl_parsers, separator).map(|decls: Vec<Decl>| {
        let mut theory = Theory::default();
        for decl in decls {
            match decl {
                Decl::Type(ty) => theory.types.push(ty),
                Decl::Oper(op) => theory.opers.push(op),
                Decl::Equation(eq) => theory.eqs.push(eq),
            }
        }
        theory
    })
}

#[test]
fn test_theory_parser() {
    use crate::combine::EasyParser;

    let theory_example =
        "#sort Bool \n#sort Int \n#func not: Bool -> Bool \n\n#rule a: Bool | not![a] = a";
    // let theory_example = "#sort Bool  \n#func not: Bool -> Bool \n#rule a: Bool | not![a] = a\n#sort Int";

    // let theory_example = "#sort Bool\n#";
    // let theory_example = "#sort Bool\n#sort Int\n#func qot: Bool -> Bool";
    // let theory_example = "#sort Bool\n#func qot: Bool -> Bool";
    // let theory_example = "#sort Bool\n#rule a: Bool | not![a] = a";
    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();
    let r = theory_parser(&types, &opers, &ctxts).easy_parse(theory_example);
    match r {
        Ok((theory, _)) => {
            dbg!(&theory);
        }
        Err(err) => {
            dbg!(&err.position);
            panic!("Failed to parse theory: {}", &err);
        }
    }
}

#[test]
fn test_theory_parser2() {
    use combine::easy::Stream;
    use combine::EasyParser;
    let types = SymbolTable::<TypeId>::new();
    let opers = SymbolTable::<OperId>::new();
    let ctxts = CtxtTable::new();
    // dbg!(&ctxts);
    let f = "theory/test.theory";
    let theory_example = std::fs::read_to_string(f).expect("Failed to read");
    let r =
        theory_parser::<Stream<&str>>(&types, &opers, &ctxts).easy_parse(theory_example.as_ref());
    // dbg!(&types);
    // dbg!(&opers);
    dbg!(&ctxts);
    dbg!(&r);
    assert!(r.is_ok());
}
