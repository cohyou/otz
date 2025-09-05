use crate::{
    context::Context,
    rule::Rule,
    subst::Subst,
    subterm::{Position, Subterm},
    symbol_table::Names,
    term::TermInner,
    unify::unify,
};

#[derive(Debug)]
pub struct Overlap {
    pub context: Context,
    pub names: Names,
    pub overlapper: Rule, // 重なる側
    pub overlappee: Rule, // 重なられる側
    pub pos: Position,
    pub subst: Subst,
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
            .filter_map(|subterm: Subterm| {
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
                        let (s, t) = (s1_sub.inner.clone(), s2.inner.clone());
                        // dbg!(is_same_rule, &s, &t);
                        unify(s, t).map(|theta| (subterm.pos, theta))
                        // .inspect(|r| {
                        //     dbg!(r);
                        // })
                    })
                    .flatten()
            })
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[case(
        "x1: Int y1: Int z1: Int | f![f![x1 y1] z1] -> f![x1 f![y1 z1]]",
        "a: Int | f![a g!a] -> 0"
    )]
    fn test_check_overlap(#[case] input_rule1: &str, #[case] input_rule2: &str) {
        use combine::Parser;

        use crate::{
            context_table::CtxtTable,
            id::VarId,
            overlap::Overlap,
            parser::rule::rule_parser,
            rule::RuleKind,
            util::{opers, types},
        };
        let types = types(vec!["Int"]);
        let opers = opers(vec!["f", "g"]);
        let ctxts = CtxtTable::new();

        let rule1 = rule_parser(&types, &ctxts, &opers).parse(input_rule1);
        let rule2 = rule_parser(&types, &ctxts, &opers).parse(input_rule2);
        dbg!(&rule1, &rule2);

        let overlap1 = rule1
            .clone()
            .unwrap()
            .0
            .check_overlap::<(VarId, usize, RuleKind)>(&rule2.clone().unwrap().0);
        let overlap2 = rule2
            .unwrap()
            .0
            .check_overlap::<(VarId, usize, RuleKind)>(&rule1.unwrap().0);
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
        "x3: Int y3: Int z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]"
    )]
    #[case(
        "x3: Int y3: Int z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]",
        "x2: Int | plus![minus!x2 x2] -> 0"
    )]
    #[case(
        "x3: Int y3: Int z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]",
        "x4: Int y4: Int z4: Int | plus![plus![x4 y4] z4] -> plus![x4 plus![y4 z4]]"
    )]
    #[case(
        "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
        "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]"
    )]
    fn test_check_overlap2(#[case] input_rule1: &str, #[case] input_rule2: &str) {
        use combine::Parser;

        use crate::{
            context_table::CtxtTable,
            id::VarId,
            overlap::Overlap,
            parser::rule::rule_parser,
            rule::RuleKind,
            util::{opers, types},
        };

        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let mut rule1 = rule_parser(&types, &ctxts, &opers)
            .parse(input_rule1)
            .clone()
            .unwrap()
            .0;
        let mut rule2 = rule_parser(&types, &ctxts, &opers)
            .parse(input_rule2)
            .clone()
            .unwrap()
            .0;
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
