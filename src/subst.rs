use std::{collections::HashMap, rc::Rc};

use crate::{
    id::VarId,
    subterm::Position,
    term::{Term, TermInner},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Subst(pub HashMap<VarId, Rc<TermInner>>);
// impl IntoIterator for &Subst {
//     type Item = <HashMap<VarId, Rc<TermInner>> as IntoIterator>::Item;
//     type IntoIter = <HashMap<VarId, Rc<TermInner>> as IntoIterator>::IntoIter;
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }
impl Subst {
    pub fn new(map: HashMap<VarId, Rc<TermInner>>) -> Self {
        map.into()
    }
    pub fn insert(&mut self, k: VarId, v: Rc<TermInner>) {
        self.0.insert(k, v);
    }
}
impl Into<Subst> for HashMap<VarId, Rc<TermInner>> {
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

pub fn substitute_inner(inner: Rc<TermInner>, var: &VarId, term: Rc<TermInner>) -> Rc<TermInner> {
    match inner.as_ref() {
        TermInner::Var(varid) if varid == var => term,
        TermInner::Fun(oper_id, args) => Rc::new(TermInner::Fun(
            oper_id.clone(),
            args.iter()
                .map(|arg| substitute_inner(arg.clone(), var, term.clone()))
                .collect(),
        )),
        _ => inner,
    }
}

impl Term {
    /// 項selfの部分項self/atをtoで置き換えた項`self[ at <- to ]`を得る
    pub fn replace(&self, at: &Position, to: Rc<Term>) -> Rc<Term> {
        let applied = Term {
            context: self.context.clone(),
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
mod test {
    use crate::{context_table::CtxtTable, id::OperId, parser::term::terminner::oper::terminner_parser, symbol_table::SymbolTable};
    use rstest::*;

    #[rstest]
    #[case("x1")]
    #[case("g!x2")]
    fn test_substitute_inner(#[case] t: &str) {
        use std::collections::HashMap;

        use combine::EasyParser;

        use crate::{id::VarId, subst::Subst};

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
        subst.insert(VarId(0), inner.unwrap().0.into());

        let r = t.unwrap().0.substitute(&Subst(subst));
        dbg!(&r);
    }
}

#[test]
fn test_substitute() {
    use crate::context::Context;
    use crate::id::TypeId;
    use crate::r#type::Type;
    use crate::term::Term;
    use std::rc::Rc;

    let mut context = HashMap::new();
    context.insert(VarId(0), Type::Unary(TypeId(0)));
    let context = Context(context);
    let inner = TermInner::Var(VarId(0));
    let term = Term {
        context,
        inner: Rc::new(inner),
    };

    let mut substs = HashMap::new();
    let v = Rc::new(TermInner::Int(100));
    substs.insert(VarId(0), v);

    let result = term.substitute(&substs.into());
    dbg!(result);
}
