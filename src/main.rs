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
    use std::{collections::HashMap, rc::Rc, vec};
    use crate::{equation::Equation, id::{OperId, Symbol, VarId}, term::TermInner};
    use crate::{context::Context, id::TypeId};

    pub fn query() {
        use crate::parser::parse_instance;
        let path = "instance/i.instance";
        let instance = parse_instance(path);
        dbg!(&instance);

        use crate::eval::eval;
        use crate::eval::Query;
        
        let mut q = Query::default();
        q.0.push(query_entity());
        let queried = eval(instance, q);
        dbg!(&queried.elems);
        dbg!(&queried.data);
    }

        
    fn query_entity() -> crate::eval::QueryEntity {
        use crate::r#type::Type;
        let ret = crate::term::Term {
            context: Rc::new(Context(HashMap::new())),
            names: Rc::new(HashMap::new()),
            inner: Rc::new(
            // 10はlast: Str
            TermInner::Fun(OperId(10), vec![
                Rc::new(TermInner::Var(VarId(100)))
            ])),
        };
        crate::eval::QueryEntity {
            entity: vec![TypeId(5)],
            fr: vec![Context(HashMap::from([(VarId(100), Type::Unary(TypeId(5)))]))],
            wh: vec![query_where()],
            att: vec![(OperId(200), ret)], // Vec<(OperId, Term)>
            // keys: t -> t'
            // transform from tableau for t' to tableau for t
            keys: vec![], // Vec<(OperId, VarId, Term)>
        }
    }

    // wrkがd2のもの
    fn query_where() -> Equation {
        // let types = SymbolTable::<TypeId>::init_with(TypeId(5));
        // let opers = SymbolTable::<OperId>::init_with(OperId(8));
        // let ctxts = CtxtTable::new();
        // ctxts.vars.borrow_mut().insert(CtxtId(0), SymbolTable::<VarId>::init_with(VarId(11)));

        // let input = "e: Emp | wrk!e = d2";  // d2はVar11
        // eq(input, &types, &opers, &ctxts)

        let ctxt = HashMap::new();
        // ctxt.insert(VarId(11), Type::Unary(TypeId(6)));
        let mut names = HashMap::new();
        names.insert("e".to_string(), Symbol::Var(VarId(100)));
        names.insert("d2".to_string(), Symbol::Fun(OperId(21)));
        names.insert("wrk".to_string(), Symbol::Fun(OperId(8)));
        let left = TermInner::Fun(OperId(8), vec![
            Rc::new(TermInner::Var(VarId(100))),
        ]);
        let right = TermInner::Fun(OperId(21), vec![]);
        Equation {
            context: Rc::new(Context(ctxt)),
            names: Rc::new(names),
            left: Rc::new(left), 
            right: Rc::new(right),
        }
    }
}

fn _comp() {
    use crate::completion::complete;
    use crate::completion::eqs;
    let rules = complete(eqs(), 0);
    crate::util::dispv("result:", &rules);
}