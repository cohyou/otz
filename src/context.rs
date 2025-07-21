use std::collections::HashMap;

use crate::id::{CtxtId, VarId};
use crate::r#type::Type;

// type List<T> = Vec<T>;
// #[derive(Debug)]
#[derive(PartialEq, Clone, Default)]
pub struct Ctxt(pub HashMap<CtxtId, HashMap<VarId, Type>>);

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
