use std::rc::Rc;

use combine::parser::char::{alpha_num, spaces, string};
use combine::stream::Stream;
use combine::Parser;
use combine::{many1, sep_by};

use crate::id::{OperId, TypeId};
use crate::oper::Oper;
use crate::parser::r#type::type_unary_parser;
use crate::r#type::Type;
use crate::symbol_table::SymbolTable;

pub fn parse_elem<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,  
) -> impl Parser<Input, Output = Vec<Oper>> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    sep_by(many1(alpha_num()), spaces())
        .skip(spaces())
        .skip(string(":"))
        .skip(spaces())
        .and(type_unary_parser(types))
        .map(move |(v, t): (Vec<Vec<_>>, _)| {
            v.into_iter().map(|ename| {
                let ename: String = ename.into_iter().collect();
                let ename = ename.trim().to_string();

                let id = opers.assign(ename);
                let dom = Rc::new(Type::Unary(TypeId(4)));
                let cod = Rc::new(t.clone());
                Oper::new(id.clone(), dom, cod)
            }).collect::<Vec<Oper>>()
        })
}

#[cfg(test)]
mod tests {
    use crate::id::{OperId, TypeId};
    use crate::symbol_table::SymbolTable;
    use crate::parser::elem::parse_elem;
    use combine::EasyParser;

    #[test]
    fn test_parse_elem() {
        let types = SymbolTable::<TypeId>::new();
        types.insert("Person".to_string(), TypeId(100));
        let opers = SymbolTable::<OperId>::init_with(OperId(3));

        let ctxt_example = "e: Person";

        let r = parse_elem(&types, &opers).easy_parse(ctxt_example);
        dbg!(&r);
        assert!(r.is_ok());
    }

    #[test]
    fn test_parse_variable2() {
        let types = SymbolTable::<TypeId>::new();
        types.insert("Person".to_string(), TypeId(100));
        let opers = SymbolTable::<OperId>::init_with(OperId(3));

        let ctxt_example = "e1 e2 e3: Person";
        let r = parse_elem(&types, &opers).easy_parse(ctxt_example);
        dbg!(&r);
        assert!(r.is_ok());
    }
}

