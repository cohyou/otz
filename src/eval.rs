use crate::{id::VarId, instance::Instance, saturate::saturate, term::TermInner};

pub fn eval(instance: Instance) -> Instance {
    let saturated = saturate(instance);
    for (_ctxtid, _vars) in saturated.elems.0 {
        let terminner = TermInner::Var(VarId(0));
        terminner.substitute()
    }
    Instance::default()
}
