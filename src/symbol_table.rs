use std::cell::RefCell;
use std::collections::HashMap;

use autoincrement::AsyncIncrement;
use autoincrement::AsyncIncremental;

// #[derive(Clone)]
pub struct SymbolTable<Id: AsyncIncremental> {
    pub table: RefCell<HashMap<String, Id>>,
    generator: AsyncIncrement<Id>,
}

impl<Id: AsyncIncremental + std::fmt::Debug> std::fmt::Debug for SymbolTable<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.table.borrow())
    }
}

impl<Id: AsyncIncremental + Clone> SymbolTable<Id> {
    pub fn new() -> Self {
        SymbolTable::<Id> {
            table: RefCell::new(HashMap::new()),
            generator: Id::init(),
        }
    }

    pub fn init_with(v: Id) -> Self {
        SymbolTable::<Id> {
            table: RefCell::new(HashMap::new()),
            generator: Id::init_with(v),
        }
    }

    pub fn assign(&self, name: String) -> Id {
        self.table
            .borrow_mut()
            .entry(name)
            .or_insert_with(|| {
                let id = self.generator.pull();
                id
            })
            .clone()
    }

    pub fn insert(&self, name: String, id: Id) {
        self.table.borrow_mut().insert(name, id);
    }

    pub fn get(&self, name: &str) -> Option<Id> {
        self.table.borrow().get(name).cloned()
    }
}
