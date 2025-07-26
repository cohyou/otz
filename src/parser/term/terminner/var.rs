use combine::many1;
use combine::parser::char::alpha_num;
use combine::stream::Stream;
use combine::Parser;

use crate::context_table::CtxtTable;
use crate::term::TermInner;

pub fn terminner_var_parser<'a, Input>(
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num()).map(move |c: Vec<_>| {
        let name: String = c.into_iter().collect();
        // dbg!(ctxts);
        let varid = ctxts.var_id_from_current(name.as_ref());
        TermInner::Var(varid)
    })
}

#[test]
fn test_terminner_var_parser() {
    use crate::combine::EasyParser;
    use crate::id::VarId;

    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("a".to_string());
    // let vars = SymbolTable::<VarId>::new();
    // vars.insert("a".to_string(), VarId(0));
    let r = terminner_var_parser(&ctxts).easy_parse("a");
    assert_eq!(r, Ok((TermInner::Var(VarId(0)), "")));
}
