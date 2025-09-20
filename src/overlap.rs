use std::rc::Rc;

use crate::{
    context::Context,
    rule::Rule,
    subst::Subst,
    subterm::{Position, SubTerm},
    symbol_table::Names,
    term::{Term, TermInner},
    unify::unify,
};

#[derive(Debug)]
pub struct Overlap {
    pub context: Rc<Context>,
    pub names: Rc<Names>,
    pub overlapper: Rule, // 重なる側
    pub overlappee: Rule, // 重なられる側
    pub pos: Position,
    pub subst: Subst,
}

impl std::fmt::Display for Overlap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = writeln!(f, "Overlap {{");
        let _ = writeln!(
            f,
            "    {} が\n    {} に {:?} で重なる",
            self.overlapper, self.overlappee, self.pos
        );
        let _ = writeln!(f, "    subst: {:#?}", self.subst);
        writeln!(f, "}}")
    }
}

impl Rule {
    /// r1:s1->t1 r2:s2->t2について、次の条件を満たすとき、r2はr1に重なるという。
    ///
    /// ある出現位置u ∈ O(s1)と代入θが存在して、以下を満たす
    /// [s1/uが変数ではない and θ(s1/u) ≡ θs2]
    /// このθは、s1/uとs2の単一化代入である。
    /// ただし、r1とr2が同一の書き換え規則である（これをr1≡r2と書く）ときには u≠ε とする。
    pub fn check_overlap<T: Eq + std::hash::Hash>(&self, from: &Rule) -> Vec<Overlap> {
        // fromがselfに重なるかだけを調べる、逆は行わない
        let s1 = self.before().clone();
        let s2 = from.before().clone();

        let is_same_rule = self.id == from.id;

        s1.subterms()
            // .inspect(|subterm| {
            //     dbg!(subterm);
            // })
            .filter_map(|subterm: SubTerm| check_overlap_inner(subterm, s2.clone(), is_same_rule))
            .map(|(pos, theta)| Overlap {
                context: self.context.clone(),
                names: self.names.clone(),
                overlapper: from.clone(),
                overlappee: self.clone(),
                pos: pos,
                subst: theta,
            })
            .collect()
    }
}

fn check_overlap_inner(
    subterm: SubTerm,
    s2: Rc<Term>,
    is_same_rule: bool,
) -> Option<(Vec<usize>, Subst)> {
    let s1_sub = subterm.term;

    // s1/uが変数なら対象外
    let s1_is_not_var = !(matches!(s1_sub.inner.as_ref(), TermInner::Var(_))
        || matches!(s1_sub.inner.as_ref(), TermInner::RuledVar(_, _, _)));
    // dbg!(&s1_sub.inner, s1_is_not_var);
    // r1≡r2ならu≠ε(恒等写像=無意味な置換になってしまう)
    let is_not_identity = !(is_same_rule && subterm.pos.is_empty());
    // dbg!(s1_is_not_var, is_not_identity);
    // 上記を前提とする
    (s1_is_not_var && is_not_identity)
        .then(|| {
            // let (s, t) = (s1_sub.inner.clone(), s2.inner.clone());
            let (s, t) = (s1_sub, s2);
            // dbg!(is_same_rule, &s, &t);
            unify(s, t).map(|theta| (subterm.pos, theta))
            // .inspect(|r| {
            //     dbg!(r);
            // })
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use rstest::rstest;

    use crate::subterm::SubTerm;
    use crate::{
        context_table::CtxtTable,
        id::VarId,
        overlap::{check_overlap_inner, Overlap},
        rule::{Rule, RuleKind},
        util::{opers, tm, types},
    };

    #[test]
    fn test_check_overlap_inner() {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let s1_input = "x1 y1: Int | plus![minus!x1 plus![x1 y1]]";
        let s1 = Rc::new(tm(s1_input, &types, &opers, &ctxts));
        let subterm = SubTerm {
            main: s1.clone(),
            pos: vec![],
            term: s1,
        };
        let s2_input = "x2: Int | plus![minus!x2 x2]";
        let s2 = Rc::new(tm(s2_input, &types, &opers, &ctxts));
        let result = check_overlap_inner(subterm, s2, false);
        dbg!(&result);
    }

    #[test]
    fn test_check_overlap_self() {
        use crate::id::{OperId, TypeId};
        use crate::term::TermInner;
        use crate::util::vars;
        use std::rc::Rc;
        // plus![minus!x/1 plus![x/1 z/2]] -> z/2
        let var_x = Rc::new(TermInner::RuledVar(VarId(0), 1, RuleKind::NotSet));
        let var_z = Rc::new(TermInner::RuledVar(VarId(2), 2, RuleKind::NotSet));
        let before = Rc::new(TermInner::Fun(
            OperId(1),
            vec![
                Rc::new(TermInner::Fun(OperId(2), vec![var_x.clone()])),
                Rc::new(TermInner::Fun(OperId(1), vec![var_x, var_z.clone()])),
            ],
        ));
        let after = var_z;

        let opers = opers(vec!["plus", "minus"]);
        let ctxts = vars(vec!["x", "y", "z"]);
        let mut names = ctxts.current_var_table();
        let oper_names = opers.current_table();
        names.extend(oper_names);
        let mut c = std::collections::HashMap::new();
        c.insert(VarId(0), crate::r#type::Type::Unary(TypeId(0)));
        c.insert(VarId(1), crate::r#type::Type::Unary(TypeId(0)));
        c.insert(VarId(2), crate::r#type::Type::Unary(TypeId(0)));
        let context = crate::context::Context(c);
        let mut rule = Rule::new(Rc::new(context), Rc::new(names), before, after);
        rule.id = Some(1);
        let rule1 = rule.make_vars_ruled(RuleKind::Set1);
        let rule2 = rule.make_vars_ruled(RuleKind::Set2);
        let overlaps = rule1.check_overlap::<(VarId, usize, RuleKind)>(&rule2);
        overlaps.iter().for_each(|ol| {
            println!("{} {:?}", ol.subst, ol.pos);
        });
    }

    #[rstest]
    #[case(
        "x1 y1 z1: Int | f![f![x1 y1] z1] -> f![x1 f![y1 z1]]",
        "a: Int | f![a g!a] -> 0"
    )]
    fn test_check_overlap(#[case] input_rule1: &str, #[case] input_rule2: &str) {
        use crate::util::rl;

        let types = types(vec!["Int"]);
        let opers = opers(vec!["f", "g"]);
        let ctxts = CtxtTable::new();

        let rule1 = rl(input_rule1, &types, &opers, &ctxts);
        let rule2 = rl(input_rule2, &types, &opers, &ctxts);
        dbg!(&rule1, &rule2);

        let overlap1 = rule1.check_overlap::<(VarId, usize, RuleKind)>(&rule2);
        let overlap2 = rule2.check_overlap::<(VarId, usize, RuleKind)>(&rule1);
        dbg!(&overlap1, &overlap2);

        let critical_pairs1 = overlap1
            .iter()
            .map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        let critical_pairs2 = overlap2
            .iter()
            .map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        dbg!(critical_pairs1, critical_pairs2);
    }

    #[rstest]
    #[case("x1: Int | plus![0 x1] -> x1", "x2: Int | plus![minus!x2 x2] -> 0")]
    #[case(
        "x1: Int | plus![0 x1] -> x1",
        "x3 y3 z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]"
    )]
    #[case(
        "x3 y3 z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]",
        "x2: Int | plus![minus!x2 x2] -> 0"
    )]
    #[case(
        "x3 y3 z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]",
        "x4 y4 z4: Int | plus![plus![x4 y4] z4] -> plus![x4 plus![y4 z4]]"
    )]
    #[case(
        "x y z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
        "x y z: Int | plus![plus![x y] z] -> plus![x plus![y z]]"
    )]
    fn test_check_overlap2(#[case] input_rule1: &str, #[case] input_rule2: &str) {
        use crate::util::rl;

        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let mut rule1 = rl(input_rule1, &types, &opers, &ctxts);
        let mut rule2 = rl(input_rule2, &types, &opers, &ctxts);
        rule1.id = Some(1);
        rule2.id = Some(1);

        let overlap1 = rule1.check_overlap::<(VarId, usize, RuleKind)>(&rule2.clone());
        let overlap2 = rule2.check_overlap::<(VarId, usize, RuleKind)>(&rule1);
        dbg!(&overlap1, &overlap2);
        let critical_pairs1 = overlap1
            .iter()
            .filter_map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        let critical_pairs2 = overlap2
            .iter()
            .filter_map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        dbg!(critical_pairs1, critical_pairs2);
    }
}
