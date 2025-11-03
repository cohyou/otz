extern crate combine;

mod completion;
mod context;

mod equation;
mod id;
mod oper;

mod reduct;


mod saturate;

mod subterm;
mod term;
mod theory;
mod r#type;

pub mod context_table;
pub mod eval;
pub mod instance;
pub mod parser;
pub mod schema;
pub mod symbol_table;
pub mod util;

fn main() {
    qu::query();
}

mod qu {
    use combine::EasyParser;

    use crate::{context_table::CtxtTable, id::{OperId}, symbol_table::SymbolTable};
    use crate::{id::TypeId};

    pub fn query() {
        let types = SymbolTable::<TypeId>::new();
        let opers = SymbolTable::<OperId>::new();
        let ctxts = CtxtTable::new();

        use crate::parser::parse_instance;
        let path = "example/instance/i.instance";
        let instance = parse_instance(path);
        println!("Parsed Instance:\n{}", instance);

        use crate::eval::eval;
        use crate::eval::Query;
        
        let mut q = Query::default();
        q.0.push(query_entity_file(&types, &opers, &ctxts));
        let queried = eval(instance, q);

        println!("{}", queried);
    }

    fn query_entity_file(
        types: &SymbolTable<TypeId>,
        opers: &SymbolTable<OperId>,
        ctxts: &CtxtTable,
    ) -> crate::eval::QueryEntity {
        use std::fs;
        use crate::parser::query::query_entity_parser;
        let path = "example/query/_.query";
        let input = fs::read_to_string(path).expect("Failed to read query file");

        let r = query_entity_parser::<combine::easy::Stream<&str>>(types, opers, ctxts)
        .easy_parse(input.as_ref()).unwrap().0;
        r
    }
}

fn _comp() {
    use crate::completion::complete;
    use crate::completion::eqs;
    let rules = complete(eqs(), 0);
    crate::util::dispv("result:", &rules);
}