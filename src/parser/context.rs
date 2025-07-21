use std::collections::HashMap;

use combine::parser::char::spaces;
use combine::sep_end_by;
use combine::stream::Stream;
use combine::Parser;

use crate::context::Ctxt;
use crate::id::TypeId;
use crate::parser::variable::parse_variable;
use crate::symbol_table::SymbolTable;
use crate::context_table::CtxtTable;

pub fn context_parser<'a, Input>(
    ctxts: &'a CtxtTable,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Ctxt> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let var_parser = parse_variable::<Input>(ctxts, types);
    sep_end_by(var_parser, spaces()).map(move |vss: Vec<_>| {
        // ctxts.complete();
        let mut res_vss = HashMap::new();
        vss.into_iter().for_each(|vs| {
            res_vss.extend(vs);
        });
        let ctxt_id = ctxts.generator.current();                                                 
        let mut new_ctxt = HashMap::new();
        new_ctxt.insert(ctxt_id.clone(), res_vss.clone());
        Ctxt(new_ctxt)
    })
}

#[test]
fn test_parse_context() {
    use combine::EasyParser;

    let ctxt_example = "x: Int p q: Bool";
    let ctxts = CtxtTable::new();
    let types = SymbolTable::<TypeId>::init_with(TypeId(3));
    types.insert("Bool".to_string(), TypeId(2));
    types.insert("Int".to_string(), TypeId(3));
    // let _new_ctxt_id = ctxts.generator.pull();
    let r = context_parser(&ctxts, &types).easy_parse(ctxt_example);
    dbg!(&ctxts);
    dbg!(&types);
    dbg!(&r);
    assert!(r.is_ok());
}
