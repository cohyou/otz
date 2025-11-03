use combine::{attempt, parser::char::{spaces}, sep_end_by};

use crate::{context::Context, context_table::CtxtTable, equation::Equation, eval::QueryEntity, id::{OperId, TypeId, VarId}, instance::Instance, parser::{instance_decl::instance_decl_parser, query::{for_decl::for_decl_parser, keys_decl::keys_decl_parser, return_decl::return_decl_parser, where_decl::where_decl_parser}}, symbol_table::SymbolTable, term::TermInner};
use combine::Parser;

mod for_decl;
mod keys_decl;
mod return_decl;
mod where_decl;

pub fn query_entity_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl combine::Parser<Input, Output = crate::eval::QueryEntity> + 'a
where
    Input: combine::Stream<Token = char> + 'a,
{
    #[derive(Clone)]
    enum Decl {
        #[allow(unused)]Instance(Instance),
        For(Context),
        Where(Equation),
        Attr((OperId, TermInner)),
        Keys((OperId, VarId, TermInner)),
    }

    let instance_parser = instance_decl_parser(types, opers, ctxts);

    let for_parser = for_decl_parser(types, ctxts);
    let where_parser = where_decl_parser(opers, ctxts);
    let return_parser = return_decl_parser(opers, ctxts);
    let keys_parser = keys_decl_parser(opers, ctxts);

    let decl_parsers = attempt(instance_parser.map(Decl::Instance))
        .or(attempt(for_parser.map(Decl::For)))
        .or(attempt(where_parser.map(Decl::Where)))
        .or(attempt(return_parser.map(Decl::Attr)))
        .or(keys_parser.map(Decl::Keys));

    sep_end_by(decl_parsers, spaces()).map(|decls: Vec<Decl>| {
        let mut query_entity = QueryEntity::default();

        for decl in decls {
            match decl {
                Decl::Instance(_) => {},  // opersがほしい
                Decl::For(fr) => query_entity.fr.push(fr),
                Decl::Where(eq) => {
                    query_entity.wh.push(eq);
                }
                Decl::Attr(eq) => query_entity.ret.push(eq),
                Decl::Keys(key) => query_entity.keys.push(key),
            }
        }
        query_entity
    })
}

#[cfg(test)]
mod tests {
    use crate::{context_table::CtxtTable, id::{OperId, TypeId}, parser::query::query_entity_parser, symbol_table::SymbolTable};

    #[test]
    fn test_query_entity_parser() {
        let types = SymbolTable::<TypeId>::init_with(TypeId(0));
        let opers = SymbolTable::<OperId>::init_with(OperId(0));
        let ctxts = CtxtTable::new();
        use combine::EasyParser;

        let f = "query/_.query";
        let input = std::fs::read_to_string(f).expect("Failed to read");

        let result = query_entity_parser::<combine::easy::Stream<&str>>(&types, &opers, &ctxts).easy_parse(input.as_ref());
        assert!(result.is_ok(), "Parser failed: {:?}", result);
        let (query_entity, remaining) = result.unwrap();
        assert!(remaining.is_empty(), "Parser did not consume all input");
        assert_eq!(query_entity.fr.len(), 1);
        assert_eq!(query_entity.wh.len(), 1);
        assert_eq!(query_entity.ret.len(), 1);
        assert_eq!(query_entity.keys.len(), 1);
    }
}
