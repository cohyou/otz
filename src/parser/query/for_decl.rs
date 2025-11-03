use combine::{Parser, Stream};

use crate::{context::Context, context_table::CtxtTable, id::TypeId, parser::variable::parse_variable, symbol_table::SymbolTable};

pub fn for_decl_parser<'a, Input>(
    types: &'a SymbolTable<TypeId>,    
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = Context> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    use crate::combine::parser::char::{spaces, string};

    string("#for")
        .skip(spaces())
        .with(
            parse_variable::<Input>(&types, &ctxts)
        )
        .map(|map| {
            Context(map)
        })
}

#[cfg(test)]
mod tests {
    use crate::{context_table::CtxtTable, id::TypeId, parser::query::for_decl::for_decl_parser, symbol_table::SymbolTable};
    use combine::EasyParser;

    #[test]
    fn test_for_decl_parser() {
        let input = "#for x y: Int";
        let ctxts = CtxtTable::new();
        let types = SymbolTable::<TypeId>::new();
        types.insert("Int".to_string(), TypeId(1)); // Mocking a type for testing
        let result = for_decl_parser(&types, &ctxts).easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());
    }
}   
