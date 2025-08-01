use crate::{context::Context, id::VarId, instance::Instance, saturate::saturate, term::{Term, TermInner}};

pub fn eval(instance: Instance) -> Instance {
    let saturated = saturate(instance);
    for (_ctxtid, _vars) in saturated.elems.0 {
        let term = Term {
            context: Context::default(),
            inner: std::rc::Rc::new(TermInner::Var(VarId(0))),
        };
        let substs = std::collections::HashMap::new();
        term.substitute(substs);
    }
    Instance::default()
}
