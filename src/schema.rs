use std::rc::Rc;

use crate::equation::Equation;
use crate::id::Symbol;
use crate::oper::Oper;
use crate::symbol_table::Names;
use crate::r#type::Type;
use crate::theory::Theory;

#[derive(Default, Clone, Debug)]
pub struct Schema {
    pub names: Rc<Names>,
    pub theory: Theory,
    pub entities: Vec<Type>,
    pub fkeys: Vec<Oper>,
    pub attrs: Vec<Oper>,
    pub constraints: Vec<Equation>,
}

impl std::fmt::Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // writeln!(f, "## Theory")?;
        writeln!(f, "{}", self.theory)?;

        writeln!(f, "<Schema>")?;

        for ent in &self.entities {
            if let Type::Unary(typeid) = ent {
                let tn = self.names.iter()
                    .find(|(_, sym)| sym == &&Symbol::Type(typeid.clone()));
                if let Some((nm, _)) = tn {
                    writeln!(f, "#sort {}", nm)?;
                } else {
                    writeln!(f, "{:?}", ent)?;
                }                
            }
        }

        writeln!(f, "")?;

        for fk in &self.fkeys {
            let fkname = self.names.iter()
                .find(|(_, sym)| sym == &&Symbol::Fun(fk.id.clone()));
            if let Some((nm, _)) = fkname {
                let domname = match fk.dom.as_ref() {
                    Type::Unary(tid) => {
                        let tn = self.names.iter()
                            .find(|(_, sym)| sym == &&Symbol::Type(tid.clone()));
                        if let Some((nm, _)) = tn {
                            nm.to_string()
                        } else {
                            format!("{:?}", fk.dom)
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
                let codname = match fk.cod.as_ref() {
                    Type::Unary(tid) => {
                        let tn = self.names.iter()
                            .find(|(_, sym)| sym == &&Symbol::Type(tid.clone()));
                        if let Some((nm, _)) = tn {
                            nm.to_string()
                        } else {
                            format!("{:?}", fk.cod)
                        }
                    }
                    _ => unimplemented!(), // 現在はないパターンなので
                };
                writeln!(f, "#fkey {}: {} -> {}", nm, domname, codname)?;
            } else {
                writeln!(f, "{:?}", fk)?;
            }
        }

        writeln!(f, "")?;

        for attr in &self.attrs {
            let attrname = self.names.iter()
                .find(|(_, sym)| sym == &&Symbol::Fun(attr.id.clone()));
            if let Some((nm, _)) = attrname {
                let domname = match attr.dom.as_ref() {
                    Type::Unary(tid) => {
                        let tn = self.names.iter()
                            .find(|(_, sym)| sym == &&Symbol::Type(tid.clone()));
                        if let Some((nm, _)) = tn {
                            nm.to_string()
                        } else {
                            format!("{:?}", attr.dom)
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
                let codname = match attr.cod.as_ref() {
                    Type::Unary(tid) => {
                        let tn = self.names.iter()
                            .find(|(_, sym)| sym == &&Symbol::Type(tid.clone()));
                        if let Some((nm, _)) = tn {
                            nm.to_string()
                        } else {
                            format!("{:?}", attr.cod)
                        }
                    }
                    _ => unimplemented!(), // 現在はないパターンなので
                };
                writeln!(f, "#attr {}: {} -> {}", nm, domname, codname)?;
            } else {
                writeln!(f, "{:?}", attr)?;
            }   
        }

        writeln!(f, "")?;

        for cons in &self.constraints {
            writeln!(f, "#rule {}", cons)?;
        }

        Ok(())
    }
}
