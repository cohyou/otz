use std::{collections::HashMap, rc::Rc};

use crate::{
    completion::critical_pairs::CriticalPair, equation::Equation, id::VarId, completion::rule::{Rule, RuleKind}, completion::subst::{Subst, Var}, term::{Term, TermInner}
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
        dbg!(&subst);
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
        let mut vars = std::collections::HashSet::new();
        vars.extend(p_term.vars());
        vars.extend(q_term.vars());
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

impl Equation {
    /// 変数を採番し直す
    pub fn refresh_vars(&self) -> Equation {
        let subst = HashMap::new();
        let (left_term, right_term) = (self.left_term(), self.right_term());
        let mut vars = std::collections::HashSet::new();
        vars.extend(left_term.vars());
        vars.extend(right_term.vars());
        let subst1 = vars.iter().fold(subst, |mut subst, var| {
            let len = subst.len();
            subst
                .entry(var.clone())
                .or_insert(Rc::new(TermInner::RuledVar(VarId(len), 0, RuleKind::NotSet)));
            subst
        });
        let left_ruled = left_term.substitute(&Subst(subst1.clone()));
        let right_ruled = right_term.substitute(&Subst(subst1.clone()));

        let subst = HashMap::new();
        let subst2 = vars.iter().fold(subst, |mut subst, _var| {
            let len = subst.len();
            subst
                .entry(Var::Ruled(VarId(len), 0, RuleKind::NotSet))
                .or_insert(Rc::new(TermInner::Var(VarId(len))));
            subst
        });

        Equation {
            context: self.context.clone(),
            names: self.names.clone(),
            left: left_ruled.substitute(&Subst(subst2.clone())).inner,
            right: right_ruled.substitute(&Subst(subst2.clone())).inner,
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

#[cfg(test)]
pub mod tests {
    use crate::{context_table::CtxtTable, util::{eq, opers, types}};

    #[test]
    fn test_refresh_vars() {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        let eq = eq("x y z: Int | plus![plus![z y] x] = 0", &types, &opers, &ctxts);
        println!("before: {}", eq);
        println!("after : {}", eq.refresh_vars());
    }
}