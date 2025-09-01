use std::{collections::HashMap, rc::Rc};

use crate::{id::VarId, rule::RuleKind, subst::{Subst, Var}, term::TermInner};

/// TODO: s/tのcontextの扱いを確認する
pub fn unify(s: Rc<TermInner>, t: Rc<TermInner>) -> Option<Subst> {
    use TermInner::{Fun, Int, Str, Var, RuledVar};

    match (s.as_ref(), t.as_ref()) {
        (rv1 @ RuledVar(_,_,_), rv2 @ RuledVar(_,_,_)) if rv1 == rv2 => Some(Subst::default()),
        (rv1 @ RuledVar(vid,rid,kind), u) | (u, rv1 @ RuledVar(vid,rid,kind)) => (!is_subterm_of2(rv1, u))
            .then_some(HashMap::from([(crate::subst::Var::Ruled(vid.clone(),*rid,kind.clone()), Rc::new(u.clone()))]).into()),

        // 全く同じ内容なら
        (Var(x), Var(y)) if x == y => Some(Subst::default()),
        (Int(x), Int(y)) if x == y => Some(Subst::default()),
        (Str(s1), Str(s2)) if s1 == s2 => Some(Subst::default()),

        // s,tのどちらかが変数
        // 変数をx, 他の項をuとする
        (Var(x), u) | (u, Var(x)) => (!is_subterm_of(x, u))
            .then_some(HashMap::from([(crate::subst::Var::Id(x.clone()), Rc::new(u.clone()))]).into()),
            
        // s,tが関数
        (Fun(oid_f, args_f), Fun(oid_g, args_g)) => {
            // t≡f(t1,...,tn),s≡g(s1,...,sm)とする
            // 前提: fとgが同じ関数 and 引数の数も同じ f ≡ g || m = n
            let is_same_oper = oid_f == oid_g && args_f.len() == args_g.len();

            is_same_oper
                .then(|| {
                    let init = Subst::default();

                    args_f
                        .iter()
                        .zip(args_g)
                        .try_fold(init, |mut theta, (tk, sk)| {
                            // 代入適用は演算子で定義してもいいかも theta[tk], theta[sk]
                            let tk_new = tk.substitute(&theta);
                            let sk_new = sk.substitute(&theta);
                            unify(tk_new, sk_new).map(|sigma| {
                                // 合成は演算子で定義してもいいかも theta = sigma[theta];
                                theta = sigma.compose(&theta);
                                // dbg!(&sigma, &theta);
                                theta
                            })
                        })
                })
                .flatten()
        }
        _t @ _ => {
            /*dbg!(t);*/
            None
        }
    }
}

impl Subst {
    // 代入の合成を行う
    // σ={x1:s1, ..., xn:sn}, τ={y1:t1, ..., yn:tn}
    // τσ=
    // {xi:τ[si] | xi ∈ D(σ) and xi !≡ τ[si], i=1~n} ∪
    // {yi:ti | yi ∈ D(τ) - D(σ), i=1~m}
    fn compose(&self, sigma: &Subst) -> Subst {
        // let mut tau = self.clone();
        let sigma_new = HashMap::new();
        let sig = sigma.0.iter().fold(sigma_new, |mut sig, (var, inner)| {
            // xi ∈ D(σ)
            let cond1 = match var {
                Var::Id(vid) => {
                    TermInner::Var(vid.clone()).substitute(&sigma).as_ref()
                    != &TermInner::Var(vid.clone())
                },
                Var::Ruled(vid, rid, kind) => {
                    TermInner::RuledVar(vid.clone(),*rid,kind.clone()).substitute(&sigma).as_ref()
                    != &TermInner::RuledVar(vid.clone(),*rid,kind.clone())
                }
            };
            // xi !≡ τ[si]
            let v = inner.substitute(self);
            let cond2 = match var {
                Var::Id(vid) => {
                    v.as_ref() != &TermInner::Var(vid.clone())
                },
                Var::Ruled(vid, rid, kind) => {
                    v.as_ref() != &TermInner::RuledVar(vid.clone(),*rid,kind.clone())
                },
            };
            (cond1 && cond2).then(|| {
                sig.insert(var.clone(), v);
            });

            sig
        });
        // dbg!(&sig);

        let tau_new = HashMap::new();
        let mut ta = self.0.iter().fold(tau_new, |mut ta, (var, inner)| {
            // yi ∈ D(τ) - D(σ)
            // var is in sigma's varが入っていると除く
            let cond = sigma.0.keys().position(|k| k == var).is_none();
            cond.then(|| {
                ta.insert(var.clone(), inner.clone());
            });

            ta
        });
        // dbg!(&ta);

        ta.extend(sig);
        Subst(ta)
    }
}

// impl From<Vec<(usize, TermInner)>> for Subst {
//     fn from(value: Vec<(usize, TermInner)>) -> Self {
//         let mut map = HashMap::new();
//         value.iter().map(|(var1, terminner)| {
//             (VarId(*var1), Rc::new(terminner))
//         }).for_each(|(k, v) | {
//             map.insert(k, v);
//         });

//         Subst(map)
//     }
// }
impl From<Vec<(usize, VarId)>> for Subst {
    fn from(value: Vec<(usize, VarId)>) -> Self {
        let mut map = HashMap::new();
        value
            .iter()
            .map(|(var1, var2)| (Var::Id(VarId(*var1)), Rc::new(TermInner::Var(var2.clone()))))
            .for_each(|(k, v)| {
                map.insert(k, v);
            });

        Subst(map)
    }
}
impl From<Vec<((usize, usize, RuleKind), VarId)>> for Subst {
    fn from(value: Vec<((usize, usize, RuleKind), VarId)>) -> Self {
        let mut map = HashMap::new();
        value
            .iter()
            .map(|((vid, rid, kind), var2)| (Var::Ruled(VarId(*vid), *rid, kind.clone()), Rc::new(TermInner::Var(var2.clone()))))
            .for_each(|(k, v)| {
                map.insert(k, v);
            });

        Subst(map)
    }
}
// impl std::ops::Index<Subst> for Subst {
//     type Output = Subst;
//     fn index<'a>(&'a self, index: Subst) -> &'a Self::Output {
//         &self.compose(index)
//     }
// }

/// xがuの真部分項かどうか
/// subtermsを調べていけばわかりそう
/// 早いかどうかは別
fn is_subterm_of(_vid: &VarId, _target: &TermInner) -> bool {
    // target.subterms().any(|t| t.term.as_ref() == self)
    false
}

fn is_subterm_of2(_ruledvar: &TermInner, _target: &TermInner) -> bool {
    // target.subterms().any(|t| t.term.as_ref() == self)
    false
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use combine::Parser;

    use crate::{
        context_table::CtxtTable,
        id::{OperId, VarId},
        parser::term::terminner::oper::terminner_parser,
        subst::{Subst, Var},
        symbol_table::SymbolTable,
        unify::unify,
    };

    use rstest::*;

    #[test]
    fn test_compose() {
        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        opers.assign("g".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("x0".to_string());
        ctxts.assign_to_current("x1".to_string());
        ctxts.assign_to_current("x2".to_string());

        let mut subst1 = Subst::default();
        let inner1 = terminner_parser(&ctxts, &opers).parse("g!x1");
        subst1.insert(Var::Id(VarId(0)), Rc::new(inner1.unwrap().0));
        let mut subst2 = Subst::default();
        let inner2 = terminner_parser(&ctxts, &opers).parse("g!x2");
        subst2.insert(Var::Id(VarId(1)), Rc::new(inner2.unwrap().0));
        // let subst = subst1.compose(subst2);
        let subst = subst2.compose(&subst1);
        dbg!(subst);
    }

    #[rstest]
    #[case("x1", "g![x2]")]
    #[case("x0", "g![x1]")]
    #[case("g![x1]", "x0")]
    // #[case("f![x0]", "f![g![x1]]")]
    #[case("f![x0 x1]", "f![g![x1] g![x2]]")]
    fn test_unify(#[case] t1: &str, #[case] t2: &str) {
        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        opers.assign("g".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("x0".to_string());
        ctxts.assign_to_current("x1".to_string());
        ctxts.assign_to_current("x2".to_string());
        let term1 = terminner_parser(&ctxts, &opers).parse(t1);
        let term2 = terminner_parser(&ctxts, &opers).parse(t2);

        let subst = unify(Rc::new(term2.unwrap().0), Rc::new(term1.unwrap().0));
        dbg!(&subst);
    }
}
