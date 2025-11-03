use std::collections::HashMap;

use combine::parser::char::spaces;
use combine::sep_end_by;
use combine::stream::Stream;
use combine::Parser;

use crate::context::Context;
use crate::context_table::CtxtTable;
use crate::id::TypeId;
use crate::parser::variable::parse_variable;
use crate::symbol_table::SymbolTable;

pub fn context_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Context> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let var_parser = parse_variable::<Input>(types, ctxts);
    sep_end_by(var_parser, spaces()).map(move |vss: Vec<_>| {
        let mut res_vss = HashMap::new();
        vss.into_iter().for_each(|vs| {
            res_vss.extend(vs);
        });

        Context(res_vss)
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

    let r = context_parser(&types, &ctxts).easy_parse(ctxt_example);
    dbg!(&ctxts);
    dbg!(&types);
    dbg!(&r);
    assert!(r.is_ok());
}
