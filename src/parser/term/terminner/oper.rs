use combine::parser::char::{alpha_num, spaces, string};
use combine::stream::Stream;
use combine::Parser;
use combine::{between, parser};
use combine::{many1, sep_by};

use crate::context_table::CtxtTable;
use crate::id::OperId;
use crate::parser::term::terminner::terminner_parser_;
use crate::symbol_table::SymbolTable;
use crate::term::TermInner;

pub fn terminner_oper_parser<'a, Input>(
    ctxts: &'a CtxtTable,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num())
        .skip(string("!"))
        .and(between(
            string("["),
            string("]"),
            sep_by(terminner_parser(ctxts, opers), spaces()),
        ))
        .map(|(c, v): (Vec<_>, Vec<_>)| {
            let name: String = c.into_iter().collect();
            let oper_id = opers
                .get(name.as_ref())
                .expect(format!("Oper '{}' not found in symbol table", name).as_ref());
            let args = v.into_iter().map(|t| t.into()).collect();
            TermInner::Fun(oper_id, args)
        })
}

parser! {
    pub fn terminner_parser['a, Input](
        // vars: std::rc::Rc<SymbolTable<VarId>>,
        ctxts: &'a CtxtTable,
        opers: &'a SymbolTable<OperId>
    )(Input) -> TermInner
    where [Input: Stream<Token = char>]
    {
        terminner_parser_(ctxts, opers)
    }
}

#[test]
fn test_terminner_oper_parser1() {
    use crate::combine::EasyParser;

    let opers = SymbolTable::<OperId>::new();
    opers.assign("f".to_string());
    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("a".to_string());

    let input = "f![a]";
    let result = terminner_oper_parser(&ctxts, &opers).easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}

#[test]
fn test_terminner_oper_parser2() {
    use crate::combine::EasyParser;
    let opers = SymbolTable::<OperId>::new();
    opers.insert("f".to_string(), OperId(0));
    let ctxts = CtxtTable::new();
    ctxts.assign_to_current("a".to_string());
    // let vars = std::rc::Rc::new(SymbolTable::<VarId>::new());
    // vars.insert("a".to_string(), VarId(0));

    let r = terminner_oper_parser(&ctxts, &opers).easy_parse("f![f![]]");
    dbg!(&opers);
    assert_eq!(
        r,
        Ok((
            TermInner::Fun(OperId(0), vec![TermInner::Fun(OperId(0), vec![]).into()]),
            ""
        ))
    );
}
