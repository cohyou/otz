use combine::{Parser, Stream, many1, parser::char::alpha_num};

use crate::{context_table::CtxtTable, id::{OperId, VarId}, parser::{term::terminner::oper::terminner_parser}, symbol_table::SymbolTable, term::{TermInner}};

pub fn keys_decl_parser<'a, Input>( 
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = (OperId, VarId, TermInner)> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    use crate::combine::parser::char::{spaces, string};

    // Example:
    // #keys wrk := [d -> e.wrk]
    // wrk: OperId
    // d: VarId (key)
    // e.wrk: TermInner
    string("#keys")
        .skip(spaces())
        .with(keys_oper_parser(opers).skip(spaces()).skip(string(":=").skip(spaces())))
        .skip(string("[").skip(spaces()))
        .and(keys_variable_parser(ctxts))
        .skip(spaces().skip(string("->")).skip(spaces()))
        .and(terminner_parser(ctxts, opers))
        .skip(string("]").skip(spaces()))
        .map(|((operid, varid), terminner)| {
            (operid, varid, terminner)
        })
}

fn keys_oper_parser<'a, Input>(
    opers: &'a SymbolTable<OperId>,
) -> impl Parser<Input, Output = OperId> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num())
        .map(move |name: String| {
            opers.assign(name)
        })
}

fn keys_variable_parser<'a, Input>(
    ctxts: &'a CtxtTable,
) -> impl Parser<Input, Output = VarId> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    many1(alpha_num())
        .map(move |name: String| {
            ctxts.assign_to_current(name)
        })
}