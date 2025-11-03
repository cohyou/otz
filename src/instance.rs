use std::rc::Rc;

use crate::{completion::subst::Subst, equation::Equation, oper::Oper, schema::Schema, symbol_table::Names};

#[derive(Clone, Default)]
pub struct Instance {
    pub names: Rc<Names>,
    pub schema: Schema,
    pub elems: Vec<Elem>,
    pub data: Vec<Equation>,
}

#[derive(Clone, Debug)]
pub enum Elem {
    Oper(Oper),
    Subst(Subst),
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self
            .data
            .iter()
            .map(|d| format!("{:?} = {:?}", d.left.clone(), d.right.clone()))
            .collect::<Vec<_>>();

        f.debug_struct("Instance")
            .field("schema", &self.schema)
            .field("elems", &self.elems)
            .field("data", &data)
            .finish()
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.schema)?;

        writeln!(f, "<Instance>")?;

        for elem in &self.elems {
            match elem {
                Elem::Oper(op) => {
                    let oname = self.names.iter()
                        .find(|(_, sym)| sym == &&crate::id::Symbol::Fun(op.id.clone()));
                    if let Some((nm, _)) = oname {
                        writeln!(f, "#elem {}", nm)?;
                    } else {
                        writeln!(f, "#elem {:?}", op)?;
                    }                    
                }
                Elem::Subst(subst) => {
                    writeln!(f, "#subst {}", subst)?;
                }
            }
        }

        writeln!(f, "")?;

        for eq in &self.data {
            writeln!(f, "{}", eq)?;
        }

        Ok(())
    }
}