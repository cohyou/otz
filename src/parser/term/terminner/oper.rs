use crate::id::{OperId, VarId};
use crate::parser::term::terminner::terminner_parser_;
use crate::symbol_table::SymbolTable;
use crate::term::TermInner;
use combine::parser;
use combine::parser::char::{alpha_num, spaces, string};
use combine::stream::Stream;
use combine::Parser;
use combine::{many1, sep_by};

pub fn terminner_oper_parser<'a, Input>(
    vars: &'a SymbolTable<VarId>,
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = TermInner> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num())
        .skip(string("!["))
        .and(sep_by(terminner_parser(vars, opers), spaces()))
        .skip(string("]"))
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
        vars: &'a SymbolTable<VarId>,
        opers: &'a SymbolTable<OperId>
    )(Input) -> TermInner
    where [Input: Stream<Token = char>]
    {
        terminner_parser_(vars, opers)
    }
}

#[test]
fn test_terminner_oper_parser1() {
    use crate::combine::EasyParser;
    let opers = SymbolTable::<OperId>::new();
    opers.insert("f".to_string(), OperId(0));
    let vars = SymbolTable::<VarId>::new();
    vars.insert("a".to_string(), VarId(0));

    let r = terminner_oper_parser(&vars, &opers).easy_parse("f![a]");
    dbg!(&opers);
    assert_eq!(
        r,
        Ok((
            TermInner::Fun(OperId(0), vec![TermInner::Var(VarId(0)).into()]),
            ""
        ))
    );
}

#[test]
fn test_terminner_oper_parser2() {
    use crate::combine::EasyParser;
    let opers = SymbolTable::<OperId>::new();
    opers.insert("f".to_string(), OperId(0));
    let vars = SymbolTable::<VarId>::new();
    vars.insert("a".to_string(), VarId(0));

    let r = terminner_oper_parser(&vars, &opers).easy_parse("f![f![]]");
    dbg!(&opers);
    assert_eq!(
        r,
        Ok((
            TermInner::Fun(OperId(0), vec![TermInner::Fun(OperId(0), vec![]).into()]),
            ""
        ))
    );
}
