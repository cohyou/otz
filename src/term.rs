use std::rc::Rc;

use crate::{
    context::Context,
    id::{OperId, Symbol, VarId},
    rule::{RuleId, RuleKind},
    symbol_table::Names,
};
type Link<T> = std::rc::Rc<T>;

#[derive(Clone, PartialEq)]
pub struct Term {
    pub context: Rc<Context>,
    pub names: Rc<Names>,
    pub inner: Rc<TermInner>,
}

impl std::fmt::Debug for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_inner(f, &self.inner)
    }
}

// impl std::cmp::PartialEq for Term {
//     fn eq(&self, other: &Self) -> bool {
//         self.inner == other.inner
//     }
// }

impl Term {
    fn fmt_inner(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        inner: &Rc<TermInner>,
    ) -> std::fmt::Result {
        use TermInner::*;
        match inner.as_ref() {
            Int(i) => {
                write!(f, "{}", i)
            }
            Var(vid) => {
                let v = self
                    .names
                    .iter()
                    .find(|(_, sym)| sym == &&Symbol::Var(vid.clone()));
                if let Some((nm, _)) = v {
                    write!(f, "{}", nm)
                } else {
                    write!(f, "v{:?}", vid.0)
                }
            }
            RuledVar(vid, rid, kind) => {
                let v = self
                    .names
                    .iter()
                    .find(|(_, sym)| sym == &&Symbol::Var(vid.clone()));
                if let Some((nm, _)) = v {
                    let _ = write!(f, "{}", nm);
                } else {
                    let _ = write!(f, "v{:?}", vid.0);
                }
                let _ = if kind == &RuleKind::Set2 {
                    write!(f, "''")
                } else if kind == &RuleKind::Set1 {
                    write!(f, "'")
                } else {
                    write!(f, "")
                };
                if kind == &RuleKind::NotSet {
                    write!(f, "/{}", rid)
                } else {
                    write!(f, "")
                }
            }
            Fun(operid, args) => {
                let v = self
                    .names
                    .iter()
                    .find(|(_, sym)| sym == &&Symbol::Fun(operid.clone()));
                if let Some((nm, _)) = v {
                    let _ = write!(f, "{}", nm);
                    match args.len() {
                        0 => {
                            let _ = write!(f, ";");
                        }
                        1 => {
                            let _ = write!(f, "!");
                            let _ = self.fmt_inner(f, &args[0]);
                        }
                        _ => {
                            let _ = write!(f, "![");
                            args.iter().enumerate().for_each(|(i, arg)| {
                                if i > 0 {
                                    let _ = write!(f, " ");
                                }
                                let _ = self.fmt_inner(f, arg);
                            });
                            let _ = write!(f, "]");
                        }
                    };
                    write!(f, "")
                } else {
                    let _ = write!(f, "f{:?}", operid.0);
                    match args.len() {
                        0 => {
                            let _ = write!(f, ";");
                        }
                        1 => {
                            let _ = write!(f, "!");
                            let _ = self.fmt_inner(f, &args[0]);
                        }
                        _ => {
                            let _ = write!(f, "![");
                            args.iter().enumerate().for_each(|(i, arg)| {
                                if i > 0 {
                                    let _ = write!(f, " ");
                                }
                                let _ = self.fmt_inner(f, arg);
                            });
                            let _ = write!(f, "]");
                        }
                    };
                    write!(f, "")
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(PartialEq, Clone, PartialOrd, Ord, Eq)]
pub enum TermInner {
    Var(VarId),
    Fun(OperId, Vec<Link<TermInner>>),
    Str(String),
    Int(usize),

    RuledVar(VarId, RuleId, RuleKind),

    Subst(VarId, Rc<TermInner>),
}

impl std::fmt::Debug for TermInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermInner::Var(id) => write!(f, "Var{:?}", id.0),
            TermInner::Fun(op_id, args) => write!(f, "Fun{:?}{:?}", op_id.0, args),
            TermInner::Str(s) => write!(f, "Str{:?}", s),
            TermInner::Int(i) => write!(f, "Int{:?}", i),

            TermInner::RuledVar(vid, rid, kind) => {
                if kind == &RuleKind::NotSet {
                    write!(f, "Var<{:?},{:?}>", vid.0, rid)
                } else {
                    write!(f, "Var<{:?},{:?}>(r{:?})", vid.0, kind, rid)
                }
            }

            TermInner::Subst(varid, inner) => write!(f, "Subst[{:?}->{:?}]", varid, inner),
        }
    }
}
