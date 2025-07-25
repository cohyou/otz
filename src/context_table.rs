// #[derive(Debug)]
use std::cell::RefCell;
use std::collections::HashMap;

use autoincrement::AsyncIncrement;
use autoincrement::AsyncIncremental;

// use crate::context::Ctxt;
use crate::id::{CtxtId, VarId};
use crate::symbol_table::SymbolTable;

pub struct CtxtTable {
    pub vars: RefCell<HashMap<CtxtId, SymbolTable<VarId>>>,
    pub generator: AsyncIncrement<CtxtId>,
}

impl std::fmt::Debug for CtxtTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.vars.borrow())
    }
}

impl CtxtTable {
    pub fn new() -> Self {
        CtxtTable {
            vars: RefCell::new(HashMap::new()),
            generator: CtxtId::init_with(CtxtId(1)),
        }
    }

    pub fn init_with(v: CtxtId) -> Self {
        CtxtTable {
            vars: RefCell::new(HashMap::new()),
            generator: CtxtId::init_with(v),
        }
    }

    pub fn assign_to_current(&self, name: String) -> VarId {
        let current_ctxt_id = self.generator.current();
        self.vars
            .borrow_mut()
            .entry(current_ctxt_id.clone())
            .or_insert_with(|| SymbolTable::<VarId>::init_with(VarId(0)))
            .assign(name)
    }

    pub fn var_id_from_current(&self, name: &str) -> VarId {
        let current_ctxt_id = self.generator.current();
        self.vars
            .borrow()
            .get(&current_ctxt_id)
            .and_then(|table| table.get(name))
            .expect(format!("Variable '{}' not found in current context", name).as_str())
    }

    pub fn complete(&self) {
        self.generator.pull();
    }
}
