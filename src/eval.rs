use std::{collections::HashMap, rc::Rc};

use crate::{
    context::Context,
    equation::Equation,
    id::{OperId, TypeId, VarId},
    instance::Instance,
    subst::{Subst, Var},
    term::{Term, TermInner},
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
    let _substs = query.0.iter().map(|query_entity| {
        eval_generators(&saturated, query_entity);
    }).inspect(|subst| { dbg!(&subst); }).collect::<Vec<_>>();
    
    // elementsをContextに変換してからInstanceに渡す

    // attの処理

    // foreign keyの処理

    Instance {
        schema: instance.schema,
        elems: Context::default(),
        data: vec![],
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
                .0
                .iter().filter_map(|e| {
                    // frからsubstを作る
                    let init = HashMap::new();
                    // for句のそれぞれのentityについて
                    let subst = context
                        .0
                        .iter().try_fold(init, |mut subst, (varid, tp)| {
                            (e.1 == tp).then(|| {
                                subst.insert(
                                    Var::Id(varid.clone()),
                                    Rc::new(TermInner::var(e.0.clone())),
                                );
                                subst
                            })
                        })
                        .map(|m| Subst::new(m));
                    // dbg!(&subst);
                    subst
                })
                .collect::<Vec<_>>();
            // dbg!(&substs);

            // 一回Vecにせずに繋げた方が効率的だが分かりやすさのため一時的にこうする
            let substs = substs
                .iter()
                .filter(|subst| {
                    // すべての等式を満たす必要がある(all)
                    query_entity.wh.iter().all(|eq| {
                        let left_substed = eq.left_term().substitute(&subst);
                        let right_substed = eq.right_term().substitute(&subst);
                        // println!("left_substed: {}", left_substed);
                        // println!("right_substed: {}", right_substed);
                        let substituted_equation = Equation {
                            context: left_substed.context,
                            names: left_substed.names,
                            left: left_substed.inner,
                            right: right_substed.inner,
                        };

                        // substituted_equationがsaturatedから導けるかどうか
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
