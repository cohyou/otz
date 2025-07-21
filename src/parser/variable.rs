use std::collections::HashMap;

use combine::parser::char::{alpha_num, spaces, string};
use combine::stream::Stream;
use combine::Parser;
use combine::{many1, sep_by};

use crate::id::{TypeId, VarId};
use crate::parser::r#type::type_unary_parser;
use crate::r#type::Type;
use crate::symbol_table::SymbolTable;
use crate::context_table::CtxtTable;

pub fn parse_variable<'a, Input>(
    ctxts: &'a CtxtTable,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = HashMap<VarId, Type>> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    sep_by(many1(alpha_num()), spaces())
        .skip(spaces())
        .skip(string(":"))
        .skip(spaces())
        .and(type_unary_parser(types))
        .map(move |(v, t): (Vec<Vec<_>>, _)| {
            let mut vartypes = HashMap::new();
            v.into_iter().for_each(|vname| {
                let vname: String = vname.into_iter().collect();
                let vname = vname.trim().to_string();
                let id = ctxts.assign_to_current(vname);
                // dbg!(ctxts);
                vartypes.insert(id, t.clone());
            });
            vartypes
        })
}

#[test]
fn test_parse_variable() {
    use combine::EasyParser;

    let ctxt_example = "x: Int";
    let ctxts = CtxtTable::new();
    let types = SymbolTable::<TypeId>::new();
    types.insert("Int".to_string(), TypeId(3)); // Mocking a type for testing
    let r = parse_variable(&ctxts, &types).easy_parse(ctxt_example);
    // dbg!(vars);
    dbg!(&r);
    assert!(r.is_ok());
}

#[test]
fn test_parse_variable2() {
    use combine::EasyParser;

    let ctxt_example = "x y z: Int";
    let ctxts = CtxtTable::new();
    let types = SymbolTable::<TypeId>::new();
    types.insert("Int".to_string(), TypeId(3));
    let r = parse_variable(&ctxts, &types).easy_parse(ctxt_example);
    dbg!(&r);
    // dbg!(&vars);
    assert!(r.is_ok());
}
