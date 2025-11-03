use std::{collections::HashMap, rc::Rc};

use crate::{
    completion::subst::{Subst, Var}, context::Context, equation::Equation, id::{OperId, TypeId, VarId}, instance::{Elem, Instance}, term::{Term, TermInner}
};

/// A tableau over a schema S is a pair of:
/// • a context over S, called the for clause, fr and
/// • a set of quantifier-free equations between terms in Terms(S,fr), called the where clause wh.

/// An uber-flower S →T consists of, for each entity t ∈ T:
/// • a tableau (fr(t), wh(t)) over S and,
/// • for each attribute att: t→t′∈T, a term [att] in Termst′ (S,frt), called the return clause for att, and
/// • for each foreign key fk: t→t′∈T, a transform [fk] from the tableau for t′to the tableau for t (note
/// the reversed direction), called the keys clause for fk,
/// • such that an equality-preservation condition holds. We defer a description of this condition until
/// section 4.3.1.

#[derive(Default)]
pub struct Query(pub Vec<QueryEntity>);

#[derive(Default, Debug)]
pub struct QueryEntity {
    pub entity: Vec<TypeId>,
    pub fr: Vec<Context>,
    pub wh: Vec<Equation>,
    pub att: Vec<(OperId, Term)>,
    // keys: t -> t'
    // transform from tableau for t' to tableau for t
    pub keys: Vec<(OperId, VarId, Term)>,
}

pub fn eval(instance: Instance, query: Query) -> Instance {
    let saturated = instance.saturate();

    // generator
    let substs = query.0.iter().map(|query_entity| {
        eval_generators(&saturated, query_entity)
    })
    // .inspect(|subst| { dbg!(&subst); })
    .collect::<Vec<_>>();

    let elems = substs.iter().flatten().map(|s| {
        Elem::Subst(s.clone())
    }).collect::<Vec<_>>();

    // attの処理
    let attrs = query.0.iter().map(|qe| {
        qe.att.iter().map(|(operid, term)| {
            // dbg!(operid, term);
            let attrs = elems.iter().filter_map(|e| {
                if let Elem::Subst(subst) = e {
                    let substed_term = term.substitute(subst);
                    let subst_vec = subst.0.iter().map(|(varid, inner)| {
                        (varid.clone(), inner.clone())
                    }).collect::<Vec<_>>();

                    // TODO: to_ruleが左->右であることを仮定している
                    let rules = instance.data.iter().map(Equation::to_rule).collect::<Vec<_>>();
                    let right = substed_term.normalize(&rules);
                    Some(Equation {
                        context: substed_term.context.clone(),
                        names: substed_term.names.clone(),
                        left: Rc::new(TermInner::Fun(
                            operid.clone(),
                            vec![
                                Rc::new(TermInner::Subst(subst_vec))
                            ],
                        )),
                        right: right.inner.clone(),
                    })
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            // dbg!(&attrs);
            attrs
        }).flatten().collect::<Vec<_>>()
    }).flatten().collect::<Vec<_>>();
    
    // foreign keyの処理
    let keys = query.0.iter().map(|qe| {
        qe.keys.iter().map(|(operid, varid, term)| {
            let keys = elems.iter().filter_map(|e| {
                if let Elem::Subst(subst) = e {
                    let substed_term = term.substitute(subst);
                    let subst_vec = subst.0.iter().map(|(varid, inner)| {
                        (varid.clone(), inner.clone())
                    }).collect::<Vec<_>>();

                    let keys_map = HashMap::from([(
                        Var::Id(varid.clone()),
                        term.inner.clone(),
                    )]);
                    let keys_subst = Subst::new(keys_map);
                    let right_subst = subst.compose(&keys_subst);
                    let right_subst_vec = right_subst.0.iter().map(|(varid, inner)| {
                        let term = Term {
                            context: Rc::new(Context(HashMap::new())),
                            names: Rc::new(HashMap::new()),
                            inner: inner.clone(),
                        };
                        // TODO: to_ruleが左->右であることを仮定している
                        let rules = instance.data.iter().map(Equation::to_rule).collect::<Vec<_>>();
                        let inner = term.normalize(&rules).inner.clone();
                        (varid.clone(), inner)
                    }).collect::<Vec<_>>();

                    Some(Equation {
                        context: substed_term.context.clone(),
                        names: substed_term.names.clone(),
                        left: Rc::new(TermInner::Fun(
                            operid.clone(),
                            vec![
                                Rc::new(TermInner::Subst(subst_vec))
                            ],
                        )),
                        right: Rc::new(TermInner::Subst(right_subst_vec)),
                    })
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            keys
        }).flatten().collect::<Vec<_>>()
    }).flatten().collect::<Vec<_>>();
                    
    let data = [attrs, keys].concat();

    Instance {
        schema: instance.schema,
        elems: elems,
        data: data,
    }
}

/// define the generators of entity tin eval(Q)(I) to be those I_EA environments for fr(t) which satisfy wh(t).
/// t: entity
/// fr(t) := {−−−→ v_i : s_i}:
/// eval(Q)(I)(t) := { [−−−−→v_i→e_i] | I⊢eq[−−−−→v_i→e_i], ∀eq ∈ wh(t), ∀e_i ∈ I_EA(s_i)}
fn eval_generators(instance: &Instance, query_entity: &QueryEntity) -> Vec<Subst> {
    query_entity
        .fr
        .iter()
        .map(|context| {
            let substs = instance
                .elems
                .iter().filter_map(|e| {
                    // frからsubstを作る
                    let init = HashMap::new();
                    // for句のそれぞれのentityについて
                    let subst = context
                        .0
                        .iter().try_fold(init, |mut subst, (varid, tp)| {
                            if let Elem::Oper(e) = e {
                                (e.cod.as_ref() == tp).then(|| {
                                    subst.insert(
                                        Var::Id(varid.clone()),
                                        Rc::new(TermInner::Fun(e.id.clone(), vec![])),
                                    );
                                    subst
                                })
                            } else {
                                None
                            }

                        })
                        .map(|m| Subst::new(m));
                    subst
                })
                .collect::<Vec<_>>();

            // 一回Vecにせずに繋げた方が効率的だが分かりやすさのため一時的にこうする
            let substs = substs
                .iter()
                .filter(|subst| {
                    // すべての等式を満たす必要がある(all)
                    query_entity.wh.iter().all(|eq| {
                        let left_substed = eq.left_term().substitute(&subst);
                        let right_substed = eq.right_term().substitute(&subst);

                        let substituted_equation = Equation {
                            context: left_substed.context,
                            names: left_substed.names,
                            left: left_substed.inner,
                            right: right_substed.inner,
                        };
                        println!("substituted_equation: {}", substituted_equation);
                        // substituted_equationがsaturatedから導けるかどうか
                        // TODO: 同じ型だとcompleteが複数回走るので効率が悪い
                        instance.deducible(&substituted_equation)
                    })
                })
                .cloned()
                .collect::<Vec<_>>();
            substs
        })
        .flatten()
        .collect()
}

impl TermInner {
    pub fn var(varid: VarId) -> Self {
        TermInner::Var(varid)
    }
}
