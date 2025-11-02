// use std::{cell::RefCell, collections::HashMap};

use combine::{
    parser::char::{spaces, string},
    Parser, Stream,
};

use crate::{
    // context_table::CtxtTable,
    id::{OperId, TypeId}, oper::Oper, parser::{DIRECTIVE_SIGN, elems::elems_parser}, symbol_table::SymbolTable,
};

pub fn elem_decl_parser<'a, Input>(
    // elems: &'a RefCell<HashMap<VarId, Type>>,
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Vec<Oper>> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    string(DIRECTIVE_SIGN)
        .and(string("elem"))
        .and(spaces())
        .with(elems_parser(types, opers))
        // .map(move |opers| {
        //     let elems = {
        //         let mut elems_ref = elems.borrow_mut();
        //         elems_ref.extend(ctxt.0);
        //         elems
        //     };

        //     elems.borrow().clone()
        // })
}

#[cfg(test)]
mod tests {
    use crate::id::{OperId, TypeId};
    use crate::symbol_table::SymbolTable;
    use crate::parser::elem_decl::elem_decl_parser;
    use combine::EasyParser;

    #[test]
    fn test_parse_elem_decl() {
        let types = SymbolTable::<TypeId>::new();
        types.insert("Emp".to_string(), TypeId(8));
        let opers = SymbolTable::<OperId>::init_with(OperId(3));

        let input = "#elem e1 e2 e3 e4 e5 e6 e7: Emp";
        let result = elem_decl_parser(&types, &opers).easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());
    }
}
