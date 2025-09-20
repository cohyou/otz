use std::{collections::HashMap, rc::Rc};

use crate::{
    critical_pairs::CriticalPair,
    id::VarId,
    rule::Rule,
    subst::Subst,
    term::{Term, TermInner},
};

impl Term {
    /// 変数を採番し直す
    pub fn refresh_vars(&self) -> Term {
        let subst = HashMap::new();
        let vars = self.vars();
        let subst = vars.iter().fold(subst, |mut subst, var| {
            let len = subst.len();
            subst
                .entry(var.clone())
                .or_insert(Rc::new(TermInner::Var(VarId(len))));
            subst
        });
        Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.substitute(&Subst(subst.clone())).inner,
        }
    }
}

impl CriticalPair {
    /// 変数を採番し直す
    pub fn refresh_vars(&self) -> CriticalPair {
        let subst = HashMap::new();
        let (p_term, q_term) = (self.p_term(), self.q_term());
        let (p_term_vars, q_term_vars) = (p_term.vars(), q_term.vars());
        let vars = if p_term_vars.len() > q_term_vars.len() {
            p_term_vars
        } else {
            q_term_vars
        };
        let subst = vars.iter().fold(subst, |mut subst, var| {
            let len = subst.len();
            subst
                .entry(var.clone())
                .or_insert(Rc::new(TermInner::Var(VarId(len))));
            subst
        });

        CriticalPair {
            context: self.context.clone(),
            names: self.names.clone(),
            p: p_term.substitute(&Subst(subst.clone())).inner,
            q: q_term.substitute(&Subst(subst.clone())).inner,
        }
    }
}

impl Rule {
    /// 変数を採番し直す
    pub fn refresh_vars(&self) -> Rule {
        // dbg!(self);
        let subst = HashMap::new();
        let vars = self.before().vars();
        let subst = vars.iter().fold(subst, |mut subst, var| {
            let len = subst.len();
            subst
                .entry(var.clone())
                .or_insert(Rc::new(TermInner::Var(VarId(len))));
            subst
        });
        // dbg!(&subst);

        let r = Rule {
            id: self.id.clone(),
            context: self.context.clone(),
            names: self.names.clone(),
            before: self.before().substitute(&Subst(subst.clone())).inner,
            after: self.after().substitute(&Subst(subst.clone())).inner,
        };
        // println!("{}", r);
        r
    }
}
