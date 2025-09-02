use crate::{
    context_table::CtxtTable,
    id::{OperId, TypeId},
    symbol_table::SymbolTable,
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
