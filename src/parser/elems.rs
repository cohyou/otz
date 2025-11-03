use combine::parser::char::spaces;
use combine::sep_end_by;
use combine::stream::Stream;
use combine::Parser;

use crate::id::{OperId, TypeId};
use crate::instance::Elem;
use crate::parser::elem::parse_elem;
use crate::symbol_table::SymbolTable;

pub fn elems_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = Vec<Elem>> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    let var_parser = parse_elem::<Input>(types, opers);
    sep_end_by(var_parser, spaces()).map(move |vss: Vec<_>| {
        let mut res_vss = vec![];
        vss.into_iter().for_each(|vs| {
            res_vss.extend(vs);
        });

        res_vss
    })
}

#[cfg(test)]
mod tests {
    use crate::id::{OperId, TypeId};
    use crate::symbol_table::SymbolTable;
    use crate::parser::elems::elems_parser;
    use combine::EasyParser;

    #[test]
    fn test_parse_context() {
        let types = SymbolTable::<TypeId>::init_with(TypeId(3));
        types.insert("Bool".to_string(), TypeId(2));
        types.insert("Int".to_string(), TypeId(3));
        let opers = SymbolTable::<OperId>::init_with(OperId(3));

        let ctxt_example = "x: Int p q: Bool";

        let r = elems_parser(&types, &opers).easy_parse(ctxt_example);
        dbg!(&opers);
        dbg!(&types);
        dbg!(&r);
        assert!(r.is_ok());
    }
}