use std::{cell::RefCell, collections::HashMap};

use combine::{
    parser::char::{spaces, string},
    Parser, Stream,
};

use crate::{
    context::Ctxt,
    context_table::CtxtTable,
    equation::Equation,
    id::{CtxtId, OperId, VarId},
    parser::{term::terminner::oper::terminner_parser, DIRECTIVE_SIGN},
    r#type::Type,
    symbol_table::SymbolTable,
};

pub fn data_decl_parser<'a, Input>(
    elems: &'a RefCell<HashMap<VarId, Type>>,
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Equation> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let left_parser = terminner_parser(ctxts, opers);
    let right_parser = terminner_parser(ctxts, opers);

    string(DIRECTIVE_SIGN)
        .and(string("data"))
        .and(spaces())
        .with(left_parser.skip(spaces()).skip(string("=").skip(spaces())))
        .and(right_parser)
        .map(|(left, right)| -> Equation {
            let mut elems_map = HashMap::new();
            elems_map.insert(CtxtId::default(), elems.borrow().clone());
            let context = Ctxt(elems_map);
            Equation {
                context,
                left,
                right,
            }
        })
}

#[test]
fn test_data_decl_parser() {
    use crate::id::TypeId;
    use combine::EasyParser;

    let input = "#data wrk![e1] = d3";

    let ctxts = CtxtTable::new();
    let elems = RefCell::new(HashMap::new());

    let var_id = ctxts.assign_to_current("e1".to_string());
    elems.borrow_mut().insert(var_id, Type::Unary(TypeId(0)));
    let var_id = ctxts.assign_to_current("d3".to_string());
    elems.borrow_mut().insert(var_id, Type::Unary(TypeId(0)));

    let opers = SymbolTable::<OperId>::new();
    opers.assign("wrk".to_string());

    let result = data_decl_parser(&elems, &ctxts, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
