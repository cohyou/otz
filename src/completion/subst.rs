use std::{collections::HashMap, rc::Rc};

use crate::{
    id::VarId,
    reduct::Redex,
    completion::rule::{RuleId, RuleKind},
    subterm::Position,
    term::{Term, TermInner},
};
use std::hash::Hash;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Var {
    Id(VarId),
    Ruled(VarId, RuleId, RuleKind),
}

impl std::fmt::Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Var::Id(vid) => write!(f, "{:?}", vid),
            Var::Ruled(v, r, k) => write!(f, "Ruled({:?},{:?},{:?})", v, r, k),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Subst(pub HashMap<Var, Rc<TermInner>>);

impl std::fmt::Display for Subst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().for_each(|(k, v)| {
            let _ = writeln!(f, "{:?}: {:?}", k, v);
        });
        write!(f, "")
    }
}

impl Subst {
    pub fn new(map: HashMap<Var, Rc<TermInner>>) -> Self {
        map.into()
    }
    pub fn insert(&mut self, k: Var, v: Rc<TermInner>) {
        self.0.insert(k, v);
    }
}
impl Into<Subst> for HashMap<Var, Rc<TermInner>> {
    fn into(self) -> Subst {
        Subst(self)
    }
}
impl std::default::Default for Subst {
    fn default() -> Self {
        HashMap::new().into()
    }
}

impl Term {
    pub fn substitute(&self, subst: &Subst) -> Term {
        Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.inner.substitute(subst),
        }
    }
}

impl TermInner {
    pub fn substitute(&self, subst: &Subst) -> Rc<TermInner> {
        let mut result = Rc::new(self.clone());
        for (var, term) in &subst.0 {
            result = substitute_inner(result.clone(), var, term.clone());
        }
        result
    }
}

/// innerの中の変数varをtermに置き換える
pub fn substitute_inner(inner: Rc<TermInner>, var: &Var, term: Rc<TermInner>) -> Rc<TermInner> {
    // println!("{:?} {:?}", inner, var);
    match (inner.as_ref(), var) {
        (TermInner::Var(vid), Var::Id(v)) if vid == v => term,
        (TermInner::RuledVar(vid, rid, kind), Var::Ruled(v, r, k))
            if vid == v && rid == r && kind == k =>
        {
            term
        }
        (TermInner::Fun(oper_id, args), _) => Rc::new(TermInner::Fun(
            oper_id.clone(),
            args.iter()
                .map(|arg| substitute_inner(arg.clone(), var, term.clone()))
                .collect(),
        )),
        _ => inner,
    }
}

impl Redex {
    /// 項termの部分項self/atをtoで置き換えた項`self[ at <- to ]`を得る。
    pub fn apply(&self) -> Rc<Term> {
        let to = self.rule.after().substitute(&self.subst);
        self.term.replace(&self.pos, to.into())
    }
}

impl Term {
    /// 項selfの部分項self/atをtoで置き換えた項`self[ at <- to ]`を得る
    pub fn replace(&self, at: &Position, to: Rc<Term>) -> Rc<Term> {
        // println!("replace: {} at: {:?} to: {}", self, at, to);
        let applied = Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: replace_term_inner(self.inner.clone(), &at, to, vec![]),
        };
        Rc::new(applied)
    }
}

/// 項selfの部分項self/atをtoで置き換えた項`self[ at <- to ]`を得る
fn replace_term_inner(
    inner: Rc<TermInner>,
    at: &Position,
    to: Rc<Term>,
    current: Position,
) -> Rc<TermInner> {
    if &current == at {
        to.inner.clone()
    } else {
        match inner.as_ref() {
            TermInner::Fun(oid, args) => {
                let applied_args = args
                    .iter()
                    .enumerate()
                    .map(|(idx, arg)| {
                        let mut arg_pos = current.clone();
                        arg_pos.push(idx);
                        replace_term_inner(arg.clone(), at, to.clone(), arg_pos)
                    })
                    .collect();
                Rc::new(TermInner::Fun(oid.clone(), applied_args))
            }
            _ => inner,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context_table::CtxtTable, id::{OperId, VarId}, parser::term::terminner::oper::terminner_parser, completion::subst::{Subst}, symbol_table::SymbolTable, util::{opers, tm, types}
    };
    use rstest::*;

    #[test]
    fn test_substitute_inner2() {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        let term = tm("x y z: Int | plus![plus![z y] x]", &types, &opers, &ctxts);
        let subst = vec![
            (1, VarId(1)),
            (0, VarId(2)),
            (2, VarId(0)),
        ];
        println!("start : {:?}", term.inner);
        println!("result: {:?}", term.inner.substitute(&Subst::from(subst)));
    }

    #[rstest]
    #[case("x1")]
    #[case("g!x2")]
    fn test_substitute_inner(#[case] t: &str) {
        use std::collections::HashMap;

        use combine::EasyParser;

        use crate::{
            id::VarId,
            completion::subst::{Subst, Var},
        };

        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        opers.assign("g".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("x0".to_string());
        ctxts.assign_to_current("x1".to_string());
        ctxts.assign_to_current("x2".to_string());
        let t = terminner_parser(&ctxts, &opers).easy_parse(t);

        let mut subst = HashMap::new();
        let inner = terminner_parser(&ctxts, &opers).easy_parse("g!x1");
        subst.insert(Var::Id(VarId(0)), inner.unwrap().0.into());

        let r = t.unwrap().0.substitute(&Subst(subst));
        dbg!(&r);
    }
}
