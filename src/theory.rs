use std::rc::Rc;

use crate::equation::Equation;
use crate::id::Symbol;
use crate::oper::Oper;
use crate::symbol_table::Names;
use crate::r#type::Type;

#[derive(Default, Clone, Debug)]
pub struct Theory {
    pub names: Rc<Names>,
    pub types: Vec<Type>,
    pub opers: Vec<Oper>,
    pub eqs: Vec<Equation>,
}

impl std::fmt::Display for Theory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<Theory>")?;
        // writeln!(f, "")?;

        for ty in &self.types {
            if let Type::Unary(typeid) = ty {
                let tn = self.names.iter()
                    .find(|(_, sym)| sym == &&Symbol::Type(typeid.clone()));
                if let Some((nm, _)) = tn {
                    writeln!(f, "#sort {}", nm)?;
                } else {
                    writeln!(f, "{:?}", ty)?;
                }                
            }
        }

        writeln!(f, "")?;

        for op in &self.opers {
            let on = self.names.iter()
                .find(|(_, sym)| sym == &&Symbol::Fun(op.id.clone()));
            if let Some((nm, _)) = on {
                let domname = match op.dom.as_ref() {
                    Type::Unary(tid) => {
                        let tn = self.names.iter()
                            .find(|(_, sym)| sym == &&Symbol::Type(tid.clone()));
                        if let Some((nm, _)) = tn {
                            nm.to_string()
                        } else {
                            format!("{:?}", op.dom)
                        }
                    }
                    Type::Binary(_, t1, t2) => {
                        let mut names = Vec::new();
                        for t in [t1, t2] {
                            match t.as_ref() {
                                Type::Unary(tid) => {
                                    let tn = self.names.iter()
                                        .find(|(_, sym)| sym == &&Symbol::Type(tid.clone()));
                                    if let Some((nm, _)) = tn {
                                        names.push(nm.to_string());
                                    } else {                        
                                        names.push(format!("{:?}", t));
                                    }                              
                                }    
                                _ => unimplemented!(), // 現在はないパターンなので
                            }                            
                        }
                        format!("({})", names.join(" * "))
                    }
                };
                let codname = match op.cod.as_ref() {
                    Type::Unary(tid) => {
                        let tn = self.names.iter()
                            .find(|(_, sym)| sym == &&Symbol::Type(tid.clone()));
                        if let Some((nm, _)) = tn {
                            nm.to_string()
                        } else {
                            format!("{:?}", op.cod)
                        }
                    }
                    _ => unimplemented!(), // 現在はないパターンなので
                };
                writeln!(f, "#func {}: {} -> {}", nm, domname, codname)?;
            } else {
                writeln!(f, "{:?}", op)?;
            }
        }

        writeln!(f, "")?;

        for eq in &self.eqs {
            writeln!(f, "#rule {}", eq)?;
        }
        Ok(())
    }
}