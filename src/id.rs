use autoincrement::prelude::*;

#[derive(AsyncIncremental, Debug, PartialEq, Eq, Clone)]
pub struct TypeId(pub usize);

#[derive(AsyncIncremental, Debug, PartialEq, Eq, Clone)]
pub struct OperId(pub usize);

#[derive(AsyncIncremental, Debug, PartialEq, Eq, Clone)]
pub struct VarId(pub usize);
