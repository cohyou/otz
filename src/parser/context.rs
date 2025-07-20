use crate::context::Ctxt;
use crate::id::{TypeId, VarId};
use crate::parser::r#type::type_unary_parser;
use crate::symbol_table::SymbolTable;
use combine::parser::char::alpha_num;
use combine::parser::char::spaces;
use combine::parser::char::string;
use combine::stream::Stream;
use combine::Parser;
use combine::{sep_by, sep_end_by, many1};

fn parse_variable<'a, Input>(
    vars: &'a SymbolTable<VarId>,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Ctxt> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    sep_by(many1(alpha_num()), spaces())
        .skip(spaces())
        .skip(string(":"))
        .skip(spaces())
        .and(type_unary_parser(types))
        .map(move |(v, t): (Vec<Vec<_>>, _)| {
            let mut ctxt = Ctxt(Vec::new().into());
            v.into_iter().for_each(|vname| {
                let vname: String = vname.into_iter().collect();
                let vname = vname.trim().to_string();
                vars.assign(vname);
                ctxt.0.push(t.clone());
            });
            ctxt
        })
}

#[test]
fn test_parse_variable() {
    use crate::r#type::Type;
    use combine::EasyParser;

    let ctxt_example = "x: Int";
    let vars = SymbolTable::<VarId>::new();

    let types = SymbolTable::<TypeId>::new();
    types.insert("Int".to_string(), TypeId(3)); // Mocking a type for testing
    let r = parse_variable(&vars, &types).easy_parse(ctxt_example);
    // dbg!(&ctxts);
    assert!(r.is_ok());
    let vars = r.unwrap().0 .0;
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0], Type::Unary(TypeId(3)));
}

#[test]
fn test_parse_variable2() {
    use crate::r#type::Type;
    use combine::EasyParser;

    let ctxt_example = "x y z: Int";
    let vars = SymbolTable::<VarId>::new();

    let types = SymbolTable::<TypeId>::new();
    types.insert("Int".to_string(), TypeId(3));
    let r = parse_variable(&vars, &types).easy_parse(ctxt_example);
    // dbg!(&ctxts);
    assert!(r.is_ok());
    let vars = r.unwrap().0 .0;
    assert_eq!(vars.len(), 3);
    assert_eq!(vars[0], Type::Unary(TypeId(3)));
    assert_eq!(vars[1], Type::Unary(TypeId(3)));
    assert_eq!(vars[2], Type::Unary(TypeId(3)));
}

pub fn context_parser<'a, Input>(
    vars: &'a SymbolTable<VarId>,
    types: &'a SymbolTable<TypeId>,
) -> impl Parser<Input, Output = Ctxt> + 'a
where
    Input: Stream<Token = char> + 'a,
{
    sep_end_by(parse_variable(vars, types), spaces()).map(move |v: Vec<_>| {
        let mut rr = vec![];
        v.into_iter().for_each(|c| {
            rr.extend(c.0.clone());
        });
        let rrr = Ctxt(rr.into());
        rrr
    })
}

#[test]
fn test_parse_context() {
    use crate::r#type::Type;
    use combine::EasyParser;

    let ctxt_example = "x: Int p q: Bool";
    let vars = SymbolTable::<VarId>::new();
    let types = SymbolTable::<TypeId>::init_with(TypeId(3));
    types.insert("Bool".to_string(), TypeId(2));
    types.insert("Int".to_string(), TypeId(3));
    let r = context_parser(&vars, &types).easy_parse(ctxt_example);
    dbg!(&vars);
    dbg!(&types);
    dbg!(&r);
    let vars = r.unwrap().0 .0;
    assert_eq!(vars.len(), 3);
    assert_eq!(vars[0], Type::Unary(TypeId(3)));
    assert_eq!(vars[1], Type::Unary(TypeId(2)));
    assert_eq!(vars[2], Type::Unary(TypeId(2)));
}
