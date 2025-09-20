use std::{collections::HashMap, rc::Rc};

use crate::{
    id::VarId,
    rule::RuleKind,
    subst::{Subst, Var},
    term::{Term, TermInner},
};

/// TODO: s/tのcontextの扱いを確認する
pub fn unify(s: Rc<Term>, t: Rc<Term>) -> Option<Subst> {
    use TermInner::{Fun, Int, RuledVar, Str, Var};

    // println!("unify s: {} t: {}", s, t);

    match (s.inner.as_ref(), t.inner.as_ref()) {
        (rv1 @ RuledVar(_, _, _), rv2 @ RuledVar(_, _, _)) if rv1 == rv2 => Some(Subst::default()),
        (RuledVar(vid, rid, kind), u) => (!is_subterm_of2(vid, rid, kind, t.as_ref())).then_some(
            HashMap::from([(
                crate::subst::Var::Ruled(vid.clone(), *rid, kind.clone()),
                Rc::new(u.clone()),
            )])
            .into(),
        ),
        (u, RuledVar(vid, rid, kind)) => (!is_subterm_of2(vid, rid, kind, s.as_ref())).then_some(
            HashMap::from([(
                crate::subst::Var::Ruled(vid.clone(), *rid, kind.clone()),
                Rc::new(u.clone()),
            )])
            .into(),
        ),

        // 全く同じ内容なら
        (Var(x), Var(y)) if x == y => Some(Subst::default()),
        (Int(x), Int(y)) if x == y => Some(Subst::default()),
        (Str(s1), Str(s2)) if s1 == s2 => Some(Subst::default()),

        // s,tのどちらかが変数
        // 変数をx, 他の項をuとする
        (Var(x), u) => (!is_subterm_of(x, t.as_ref())).then_some(
            HashMap::from([(crate::subst::Var::Id(x.clone()), Rc::new(u.clone()))]).into(),
        ),
        (u, Var(x)) => (!is_subterm_of(x, s.as_ref())).then_some(
            HashMap::from([(crate::subst::Var::Id(x.clone()), Rc::new(u.clone()))]).into(),
        ),

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
                            let tk_new = Rc::new(Term {
                                context: s.context.clone(),
                                names: s.names.clone(),
                                inner: tk_new,
                            });
                            let sk_new = sk.substitute(&theta);
                            let sk_new = Rc::new(Term {
                                context: t.context.clone(),
                                names: t.names.clone(),
                                inner: sk_new,
                            });
                            unify(tk_new, sk_new).map(|sigma| {
                                // 合成は演算子で定義してもいいかも theta = sigma[theta];
                                theta = sigma.compose(&theta);
                                // println!("unify sigma: {:?}\n      composed theta: {:?}", &sigma, &theta);
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
    pub fn compose(&self, sigma: &Subst) -> Subst {
        // let mut tau = self.clone();
        let sigma_new = HashMap::new();
        let sig = sigma.0.iter().fold(sigma_new, |mut sig, (var, inner)| {
            // xi ∈ D(σ)
            let cond1 = match var {
                Var::Id(vid) => {
                    TermInner::Var(vid.clone()).substitute(&sigma).as_ref()
                        != &TermInner::Var(vid.clone())
                }
                Var::Ruled(vid, rid, kind) => {
                    TermInner::RuledVar(vid.clone(), *rid, kind.clone())
                        .substitute(&sigma)
                        .as_ref()
                        != &TermInner::RuledVar(vid.clone(), *rid, kind.clone())
                }
            };
            // xi !≡ τ[si]
            let v = inner.substitute(self);
            let cond2 = match var {
                Var::Id(vid) => v.as_ref() != &TermInner::Var(vid.clone()),
                Var::Ruled(vid, rid, kind) => {
                    v.as_ref() != &TermInner::RuledVar(vid.clone(), *rid, kind.clone())
                }
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
            .map(|((vid, rid, kind), var2)| {
                (
                    Var::Ruled(VarId(*vid), *rid, kind.clone()),
                    Rc::new(TermInner::Var(var2.clone())),
                )
            })
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
fn is_subterm_of(vid: &VarId, target: &Term) -> bool {
    target.vars().contains(&Var::Id(vid.clone()))
}

fn is_subterm_of2(v: &VarId, r: &usize, k: &RuleKind, target: &Term) -> bool {
    target
        .vars()
        .contains(&Var::Ruled(v.clone(), *r, k.clone()))
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use combine::Parser;

    use crate::{
        context_table::CtxtTable,
        id::{OperId, VarId},
        parser::term::terminner::oper::terminner_parser,
        subst::{Subst, Var},
        symbol_table::SymbolTable,
        term::Term,
        unify::unify,
        util::{opers, tm, types},
    };

    use rstest::*;

    #[rstest]
    // #[case(unifing1())]
    // #[case(unifing2())]
    #[case(unifing3())]
    fn test_unify_self(#[case] (s, t): (Rc<Term>, Rc<Term>)) {
        let result = unify(s, t);
        dbg!(&result);
    }

    // use crate::term::TermInner;
    // fn unifing1() -> (Rc<TermInner>, Rc<TermInner>) {
    //     use std::rc::Rc;

    //     use crate::id::{OperId};
    //     // use crate::util::vars;
    //     use crate::rule::RuleKind;
    //     // plus![minus!x/1 plus![x/1 z/2]] -> z/2
    //     let var_x_1 = Rc::new(TermInner::RuledVar(VarId(0), 1, RuleKind::Set1));
    //     let var_z_1 = Rc::new(TermInner::RuledVar(VarId(2), 2, RuleKind::Set1));
    //     let var_x_2 = Rc::new(TermInner::RuledVar(VarId(0), 1, RuleKind::Set2));
    //     let var_z_2 = Rc::new(TermInner::RuledVar(VarId(2), 2, RuleKind::Set2));
    //     let s = Rc::new(TermInner::Fun(OperId(1), vec![var_x_1.clone(), var_z_1.clone()]));
    //     let t = Rc::new(TermInner::Fun(OperId(1), vec![
    //         Rc::new(TermInner::Fun(OperId(2), vec![var_x_2.clone()])),
    //         Rc::new(TermInner::Fun(OperId(1), vec![var_x_2.clone(), var_z_2.clone()]))
    //     ]));
    //     (s, t)
    // }

    // fn unifing2() -> (Rc<TermInner>, Rc<TermInner>) {
    //     use std::rc::Rc;

    //     use crate::id::{OperId};
    //     // use crate::util::vars;
    //     // use crate::rule::RuleKind;
    //     // plus![minus!x/1 plus![x/1 z/2]] -> z/2
    //     let var_x = Rc::new(TermInner::Var(VarId(0)));
    //     let var_z = Rc::new(TermInner::Var(VarId(2)));
    //     let s = Rc::new(TermInner::Fun(OperId(1), vec![var_x.clone(), var_z.clone()]));
    //     let t = Rc::new(TermInner::Fun(OperId(1), vec![
    //         Rc::new(TermInner::Fun(OperId(2), vec![var_x.clone()])),
    //         s.clone()
    //     ]));
    //     (s, t)
    // }

    fn unifing3() -> (Rc<Term>, Rc<Term>) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        let s1_input = "x1 y1: Int | plus![minus!x1 plus![x1 y1]]";
        let s1 = Rc::new(tm(s1_input, &types, &opers, &ctxts));
        let s2_input = "x2: Int | plus![minus!x2 x2]";
        let s2 = Rc::new(tm(s2_input, &types, &opers, &ctxts));
        (s1, s2)
    }

    #[rstest]
    #[case("x1: Int | x1", "x2: Int | g![x2]")]
    #[case("x0: Int | x0", "x1: Int | g![x1]")]
    #[case("x1: Int | g![x1]", "x0: Int | x0")]
    // #[case("f![x0]", "f![g![x1]]")]
    #[case("f![x0 x1]", "f![g![x1] g![x2]]")]
    fn test_unify(#[case] t1: &str, #[case] t2: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["f", "g"]);
        let ctxts = CtxtTable::new();
        let term1 = tm(t1, &types, &opers, &ctxts);
        let term2 = tm(t2, &types, &opers, &ctxts);

        let subst = unify(Rc::new(term2), Rc::new(term1));
        dbg!(&subst);
    }

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
}
