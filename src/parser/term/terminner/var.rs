use crate::id::VarId;
use crate::symbol_table::SymbolTable;
use crate::term::TermInner;
use combine::many1;
use combine::parser::char::alpha_num;
use combine::stream::Stream;
use combine::Parser;

pub fn terminner_var_parser<'a, Input>(
    vars: &'a SymbolTable<VarId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    // This parser will parse a variable name and return a TermInner::Var variant
    many1(alpha_num()).map(|c: Vec<_>| {
        let name: String = c.into_iter().collect();
        let varid = vars
            .get(name.as_ref())
            .expect(format!("Variable '{}' not found in symbol table", name).as_ref());
        TermInner::Var(varid)
    })
}

#[test]
fn test_terminner_var_parser() {
    use crate::combine::EasyParser;
    let vars = SymbolTable::<VarId>::new();
    vars.insert("a".to_string(), VarId(0));
    let r = terminner_var_parser(&vars).easy_parse("a");
    assert_eq!(r, Ok((TermInner::Var(VarId(0)), "")));
}
