use std::rc::Rc;

use crate::{context::Context, equation::Equation, id::{OperId, TypeId, VarId}, instance::Instance, term::{Term, TermInner}};

#[derive(Default)]
pub struct Query {
    _entity: Vec<TypeId>,
    r#_for: Vec<Context>,
    r#where: Vec<Equation>,
    _att: Vec<(OperId, Term)>,
    // keys: t -> t'
    // transform from tableau for t' to tableau for t
    _keys: Vec<(OperId, VarId, Term)>
}

pub fn eval(instance: Instance, query: Query) -> Instance {
    let saturated = instance.saturate();
    let mut elements = vec![];
    for (_varid, _type) in saturated.elems.0 {
        for eq in query.r#where.iter() {
            let substs = std::collections::HashMap::new(); // 実際にはvaridとtypeからつくる
            let left_substed = eq.left_term().substitute(substs.clone());
            let right_substed = eq.right_term().substitute(substs.clone());

            let substituted_equation = Equation {
                context: left_substed.context,
                left: left_substed.inner,
                right: right_substed.inner,
            };

            // substituted_equationがsaturatedから導けるかどうか。
            // 導くことができれば、そのsubstsをテーブルの要素としてpushする
            if instance.deducible(&substituted_equation) {
                elements.push(TermInner::Subst(VarId(0), Rc::new(TermInner::Int(0))));
            }            
        }
    }
    // elementsをContextに変換してからInstanceに渡す

    // attの処理

    // foreign keyの処理

    Instance {
        schema: saturated.schema,
        elems: Context::default(),
        data: vec![],
    }
}
