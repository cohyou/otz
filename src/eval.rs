use std::{collections::HashMap, rc::Rc};

use crate::{
    completion::subst::{Subst, Var}, context::Context, equation::Equation, id::{OperId, TypeId, VarId}, instance::{Elem, Instance}, symbol_table::Names, term::{Term, TermInner}
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
    pub fr: Context,
    pub wh: Vec<Equation>,
    pub ret: Vec<(OperId, TermInner)>,
    // keys: t -> t'
    // transform from tableau for t' to tableau for t
    pub keys: Vec<(OperId, VarId, TermInner)>,
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
        qe.ret.iter().map(|(operid, term)| {
            // dbg!(operid, term);
            let attrs = elems.iter().filter_map(|e| {
                if let Elem::Subst(subst) = e {
                    let substed_term = term.substitute(subst);
                    let subst_vec = subst.0.iter().map(|(varid, inner)| {
                        (varid.clone(), inner.clone())
                    }).collect::<Vec<_>>();

                    // TODO: to_ruleが左->右であることを仮定している
                    let rules = instance.data.iter().map(Equation::to_rule).collect::<Vec<_>>();
                    let right_term = Term {
                        context: Rc::new(Context::default()),
                        names: Rc::new(Names::default()),
                        inner: substed_term.clone(),
                    };
                    let right = right_term.normalize(&rules);
                    Some(Equation {
                        context: Rc::new(Context::default()),
                        names: instance.names.clone(),
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
                        substed_term.clone(),
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
                        context: Rc::new(Context(HashMap::new())),
                        names: instance.names.clone(),
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
        names: instance.names.clone(),
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
    // fr(t)の各変数に対して、型の合う要素を独立に割り当てた環境(直積)をすべて作る。
    let substs = query_entity.fr.0.iter().fold(
        vec![Subst::default()],
        |partials, (varid, tp)| {
            let mut next = Vec::new();
            for partial in &partials {
                for e in &instance.elems {
                    if let Elem::Oper(op) = e {
                        if op.cod.as_ref() == tp {
                            let mut m = partial.0.clone();
                            m.insert(
                                Var::Id(varid.clone()),
                                Rc::new(TermInner::Fun(op.id.clone(), vec![])),
                            );
                            next.push(Subst::new(m));
                        }
                    }
                }
            }
            next
        },
    );

    // すべての等式を満たす必要がある(all)
    substs
        .into_iter()
        .filter(|subst| {
            query_entity.wh.iter().all(|eq| {
                let left_substed = eq.left_term().substitute(subst);
                let right_substed = eq.right_term().substitute(subst);

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
        .collect()
}

impl TermInner {
    pub fn var(varid: VarId) -> Self {
        TermInner::Var(varid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oper::Oper;
    use crate::r#type::Type;

    fn make_oper_elem(id: usize, cod: &Type) -> Elem {
        Elem::Oper(Oper::new(
            OperId(id),
            Rc::new(Type::Unary(TypeId(0))),
            Rc::new(cod.clone()),
        ))
    }

    /// #for x y: Tp のように同じ型の複数変数がある場合、
    /// それぞれ独立に要素を割り当てた直積の環境がすべて得られること。
    #[test]
    fn test_eval_generators_cross_product_same_type() {
        let tp = Type::Unary(TypeId(1));
        let instance = Instance {
            names: Default::default(),
            schema: Default::default(),
            elems: vec![make_oper_elem(1, &tp), make_oper_elem(2, &tp)],
            data: vec![],
        };

        let mut fr = Context::default();
        fr.0.insert(VarId(0), tp.clone());
        fr.0.insert(VarId(1), tp.clone());

        let query_entity = QueryEntity {
            fr,
            ..Default::default()
        };

        let substs = eval_generators(&instance, &query_entity);
        // 2変数 x 2要素 = 4通りの直積になっているはず(修正前は2通りだった)
        assert_eq!(substs.len(), 4);
    }

    /// 型が異なる複数の#for行(エンティティ間のjoin)も、
    /// マージされた1つのcontextとして直積が取れること。
    #[test]
    fn test_eval_generators_cross_product_different_types() {
        let tp1 = Type::Unary(TypeId(1));
        let tp2 = Type::Unary(TypeId(2));
        let instance = Instance {
            names: Default::default(),
            schema: Default::default(),
            elems: vec![
                make_oper_elem(1, &tp1),
                make_oper_elem(2, &tp1),
                make_oper_elem(3, &tp2),
            ],
            data: vec![],
        };

        let mut fr = Context::default();
        fr.0.insert(VarId(0), tp1.clone());
        fr.0.insert(VarId(1), tp2.clone());

        let query_entity = QueryEntity {
            fr,
            ..Default::default()
        };

        let substs = eval_generators(&instance, &query_entity);
        // tp1の要素2つ x tp2の要素1つ = 2通り
        assert_eq!(substs.len(), 2);
    }
}
