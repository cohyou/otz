use crate::{
    context_table::CtxtTable,
    equation::Equation,
    id::{OperId, TypeId},
    parser::{equation::equation_parser, rule::rule_parser, term::term_parser},
    rule::Rule,
    symbol_table::SymbolTable,
    term::Term,
};

pub fn types(names: Vec<&str>) -> SymbolTable<TypeId> {
    let types = SymbolTable::<TypeId>::new();
    names.iter().for_each(|name| {
        types.assign(name.to_string());
    });
    types
}

pub fn opers(names: Vec<&str>) -> SymbolTable<OperId> {
    let opers = SymbolTable::<OperId>::new();
    names.iter().for_each(|name| {
        opers.assign(name.to_string());
    });
    opers
}

pub fn vars(names: Vec<&str>) -> CtxtTable {
    let ctxts = CtxtTable::new();
    names.iter().for_each(|name| {
        ctxts.assign_to_current(name.to_string());
    });
    ctxts
}
use combine::EasyParser;
pub fn rl<'a>(
    input: &str,
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> Rule {
    rule_parser(&types, &ctxts, &opers)
        .easy_parse(input)
        .unwrap()
        .0
}

pub fn rules<'a>(
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
    inputs: Vec<&str>,
) -> Vec<Rule> {
    inputs
        .iter()
        .map(|input| rl(input, types, opers, ctxts))
        .collect()
}

pub fn tm<'a>(
    input: &str,
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> Term {
    term_parser(&types, &opers, &ctxts)
        .easy_parse(input)
        .unwrap()
        .0
}

pub fn eq<'a>(
    input: &str,
    types: &'a SymbolTable<TypeId>,
    opers: &'a SymbolTable<OperId>,
    ctxts: &'a CtxtTable,
) -> Equation {
    equation_parser(&types, &opers, &ctxts)
        .easy_parse(input)
        .unwrap()
        .0
}

pub fn dispv<T: std::fmt::Display>(vec: &Vec<T>) {
    println!("--- {} items [", vec.len());
    vec.iter().for_each(|item| {
        println!("    {}", item);
    });
    println!("] ---");
}
