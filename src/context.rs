use std::collections::HashMap;

use crate::id::VarId;
use crate::r#type::Type;

#[derive(PartialEq, Clone, Default)]
pub struct Context(pub HashMap<VarId, Type>);

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().for_each(|(k, v)| {
            write!(f, "{:?}: {:?}", k, v).unwrap();
        });
        write!(f, "")
    }
}
