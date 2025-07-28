use std::{cell::RefCell, collections::HashMap};

use combine::{
    parser::char::{spaces, string},
    Parser, Stream,
};

use crate::{
    context_table::CtxtTable,
    id::{TypeId, VarId},
    parser::{context::context_parser, DIRECTIVE_SIGN},
    r#type::Type,
    symbol_table::SymbolTable,
};

pub fn elem_decl_parser<'a, Input>(
    elems: &'a RefCell<HashMap<VarId, Type>>,
    ctxts: &'a CtxtTable,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = HashMap<VarId, Type>> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string(DIRECTIVE_SIGN)
        .and(string("elem"))
        .and(spaces())
        .with(context_parser(ctxts, types))
        .map(move |ctxt| {
            let elems = {
                let mut elems_ref = elems.borrow_mut();
                elems_ref.extend(ctxt.0);
                elems
            };

            elems.borrow().clone()
        })
}

#[test]
fn test_parse_elem_decl() {
    use combine::EasyParser;

    let elems = RefCell::new(HashMap::new());
    let ctxts = CtxtTable::new();
    let types = SymbolTable::<TypeId>::new();
    let input = "#elem e1 e2 e3 e4 e5 e6 e7: Emp";
    let result = elem_decl_parser(&elems, &ctxts, &types).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
