use std::collections::HashMap;

use crate::id::{CtxtId, VarId};
use crate::r#type::Type;

// type List<T> = Vec<T>;
// #[derive(Debug)]
#[derive(PartialEq, Clone, Default)]
pub struct Ctxt(pub HashMap<CtxtId, HashMap<VarId, Type>>);
impl Ctxt {
    pub fn extend_to_default(&mut self, vars: HashMap<VarId, Type>) {
        self.0
            .entry(CtxtId(0))
            .and_modify(|target| {
                target.extend(vars);
            })
            .or_insert(HashMap::default());
    }
}
impl std::fmt::Debug for Ctxt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
        // write!(f, "{:?}(", self.0)?;
        // for (i, ty) in self.0.iter().enumerate() {
        //     if i > 0 {
        //         let _ = write!(f, ", ");
        //     }
        //     let _ = write!(f, "{:?}: {:?}", ty.0, ty.1);
        // }
        // write!(f, ")")
    }
}
