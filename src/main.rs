extern crate combine;

mod analyse;
mod comp3;
mod completion;
mod context;
mod critical_pairs;
mod equation;
mod id;
mod oper;
mod overlap;
mod reduct;
mod renumber;
mod rule;
mod saturate;
mod subst;
mod subterm;
mod term;
mod theory;
mod r#type;
mod unify;

pub mod context_table;
pub mod eval;
pub mod instance;
pub mod parser;
pub mod schema;
pub mod symbol_table;
pub mod util;

fn main() {
    // comp();
    // comp2();
    comp3();
}

fn _query() {
    use crate::parser::parse_instance;
    let path = "instance/i.instance";
    let instance = parse_instance(path);
    dbg!(&instance);

    use crate::eval::eval;
    use crate::eval::Query;
    let queried = eval(instance, Query::default());
    dbg!(&queried);
}

#[allow(unused)]
fn comp() {
    use crate::completion::complete;
    use crate::completion::eqs;
    let _rules = complete(&eqs(), 1);
}

#[allow(unused)]
fn comp2() {
    use crate::completion::complete2;
    use crate::completion::eqs;
    let eqs_bh = std::collections::BinaryHeap::from(eqs());
    let rules = complete2(eqs_bh, 200);
    crate::util::dispv("result:", &rules);
}

fn comp3() {
    use crate::comp3::complete3;
    use crate::completion::eqs;
    let rules = complete3(eqs(), 0);
    crate::util::dispv("result:", &rules);
}