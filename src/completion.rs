use std::{collections::HashMap, rc::Rc};

use crate::{
    analyse::analyse,
    context::Context,
    equation::Equation,
    id::VarId,
    rule::{Rule, RuleKind},
    subst::{Subst, Var},
    subterm::{Position, Subterm},
    symbol_table::Names,
    term::{Term, TermInner},
    unify::unify,
};

pub fn complete(eqs: &Vec<Equation>) -> Vec<Rule> {
    let mut rules = eqs
        .iter()
        .map(|eq| analyse(eq.context.clone(), eq.names.clone(), &eq.left, &eq.right))
        .collect::<Vec<_>>();
    let mut critical_pairs = make_critical_pair_set(&rules);

    println!("--- {}", critical_pairs.len());
    critical_pairs.iter().for_each(|r| {
        println!("{}", r);
    });
    println!("---");

    while !critical_pairs.is_empty() {
        let cp = critical_pairs.pop().unwrap();
        // p, qのrulesに関しての正規形p^,q^を求める
        let normal_p = cp.p_term().normalize(&rules);
        let normal_q = cp.q_term().normalize(&rules);
        
        if normal_p != normal_q {
            println!("{} != {}", normal_p, normal_q);

            let new_rule = analyse(cp.context, cp.names, &normal_p.inner, &normal_q.inner);
            rules.push(new_rule.clone());
            // α→βと既存rules内のrule毎の危険対の集合を作る
            let new_pairs = rules
                .iter()
                .flat_map(|rule| find_critical_pairs(&new_rule, rule))
                .collect::<Vec<_>>();
            critical_pairs.extend(new_pairs);
        } else {
            println!("{} == {}", normal_p, normal_q);
        }

        println!("--- {}", critical_pairs.len());
        critical_pairs.iter().for_each(|r| {
            println!("{}", r);
        });
        println!("---");
    }
    rules
}

struct CriticalPair {
    context: Context,
    names: Names,
    p: Rc<TermInner>,
    q: Rc<TermInner>,
}

impl CriticalPair {
    pub fn p_term(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.p.clone(),
        })
    }
    pub fn q_term(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: self.q.clone(),
        })
    }
}

impl std::fmt::Debug for CriticalPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "< {:?} | {:?} >", self.p, self.q)
    }
}

impl std::fmt::Display for CriticalPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "< {} | {} >", self.p_term(), self.q_term())
    }
}

fn prepare_rules(rules: &Vec<Rule>) -> Vec<Rule> {
    rules
        .iter()
        .enumerate()
        .map(|(idx, rule)| {
            let mut r = rule.clone();
            r.id = Some(idx);
            r
        })
        .collect()
}

impl Rule {
    pub fn make_vars_ruled(&self, kind: RuleKind) -> Rule {
        let subst = HashMap::new();
        let subst = self.context.0.keys().fold(subst, |mut subst, v| {
            let var = Var::Id(v.clone());
            let ruled_var = TermInner::RuledVar(v.clone(), self.id.unwrap(), kind.clone());
            subst.insert(var, Rc::new(ruled_var));
            subst
        });
        // dbg!(&subst);

        Rule {
            id: self.id.clone(),
            context: self.context.clone(),
            names: self.names.clone(),
            before: self.before().substitute(&Subst(subst.clone())).inner,
            after: self.after().substitute(&Subst(subst.clone())).inner,
        }
    }
}

/// TODO: 最初に変数同士のidが重ならないように変換が必要
fn make_critical_pair_set(rules: &Vec<Rule>) -> Vec<CriticalPair> {
    // それぞれ2つのruleを取り出して、find_critical_pairsする
    // 同一ruleも対象
    let rules = prepare_rules(rules);
    rules
        .iter()
        .flat_map(|rule1| {
            rules
                .iter()
                .flat_map(|rule2| {
                    let overlaps = if rule1.id == rule2.id {
                        let r1 = rule1.make_vars_ruled(RuleKind::Set1);
                        let r2 = rule2.make_vars_ruled(RuleKind::Set2);
                        // dbg!(&r1, &r2);
                        r1.check_overlap::<(VarId, usize, RuleKind)>(&r2)
                    } else {
                        let r1 = rule1.make_vars_ruled(RuleKind::NotSet);
                        let r2 = rule2.make_vars_ruled(RuleKind::NotSet);
                        // dbg!(&r1, &r2);
                        r1.check_overlap::<(VarId, usize, RuleKind)>(&r2)
                        // rule1.check_overlap::<(VarId, usize, RuleKind)>(rule2)
                    };
                    overlaps
                        .iter()
                        .filter_map(Overlap::to_critical_pair)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn find_critical_pairs(rule1: &Rule, rule2: &Rule) -> Vec<CriticalPair> {
    // rule2とrule1との互いの危険対を探す
    // rule2とrule2・rule1とrule1の組み合わせは探さない
    let overlaps1 = rule1.check_overlap::<(VarId, usize, RuleKind)>(rule2);
    let pairs1 = overlaps1.iter().filter_map(Overlap::to_critical_pair);
    let overlaps2 = rule2.check_overlap::<(VarId, usize, RuleKind)>(rule1);
    let pairs2 = overlaps2.iter().filter_map(Overlap::to_critical_pair);
    pairs1.chain(pairs2).collect()
}
#[derive(Debug)]
struct Overlap {
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
    fn check_overlap<T: Eq + std::hash::Hash>(&self, from: &Rule) -> Vec<Overlap> {
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

impl Overlap {
    /// r1:s1->t1 r2:s2->t2
    /// r2がuでmguθによりr1に重なるとする
    /// その場合<θs1[u<-t2], θt1>をr1とr2の危険対という
    /// ちなみに、一般性を失わず、s1とs2の変数は重ならないとしてよい
    fn to_critical_pair(&self) -> Option<CriticalPair> {
        // 重なりを危険対に変換する

        // θs1[u<-t2]
        let s1 = self.overlappee.before();
        let theta_s1 = s1.substitute(&self.subst);
        // dbg!(&theta_s1);
        let to = self.overlapper.after().substitute(&self.subst);
        let left = theta_s1.replace(&self.pos, Rc::new(to));

        // θt1
        let t1 = self.overlappee.after();
        let right = Rc::new(t1.substitute(&self.subst));

        // 代入の結果が同一の場合は危険対とは見做さない
        (left != right).then_some(CriticalPair {
            context: self.context.clone(),
            names: self.names.clone(),
            p: left.inner.clone(),
            q: right.inner.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use combine::Parser;

    use crate::{
        completion::{complete, make_critical_pair_set, Overlap}, context_table::CtxtTable, equation::Equation, parser::{equation::equation_parser, rule::rule_parser}, rule::{Rule, RuleKind}, util::{opers, types}
    };

    use rstest::*;

    #[test]
    fn test_complete() {
        let rules = complete(&eqs());
        rules.iter().for_each(|r| {
            println!("{}", r);
        });
    }

    fn eqs() -> Vec<Equation> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let input_rule1 = "x: Int | plus![0 x] = x";
        let input_rule2 = "x: Int | plus![minus!x x] = 0";
        let input_rule3 = "x: Int y: Int z: Int | plus![plus![x y] z] = plus![x plus![y z]]";
        let eq1 = equation_parser(&types, &opers, &ctxts)
            .parse(input_rule1)
            .unwrap()
            .0;
        let eq2 = equation_parser(&types, &opers, &ctxts)
            .parse(input_rule2)
            .unwrap()
            .0;
        let eq3 = equation_parser(&types, &opers, &ctxts)
            .parse(input_rule3)
            .unwrap()
            .0;
        vec![eq1, eq2, eq3]
    }
    #[test]
    fn test_make_critical_pair_set() {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let input_rule1 = "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]";
        let mut rule1 = rule_parser(&types, &ctxts, &opers)
            .parse(input_rule1)
            .clone()
            .unwrap()
            .0;
        rule1.id = Some(1);
        let critical_pairs = make_critical_pair_set(&vec![rule1]);
        dbg!(&critical_pairs);
        critical_pairs.iter().for_each(|cp| {
            println!("{}", cp);
        });
    }

    #[test]
    fn test_make_vars_ruled() {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let input_rule1 = "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]";
        let mut rule1 = rule_parser(&types, &ctxts, &opers)
            .parse(input_rule1)
            .clone()
            .unwrap()
            .0;
        rule1.id = Some(1);
        let rule = rule1.make_vars_ruled(RuleKind::Set2);
        println!("{}", &rule);
    }

    #[rstest]
    #[case(
        "x1: Int y1: Int z1: Int | f![f![x1 y1] z1] -> f![x1 f![y1 z1]]",
        "a: Int | f![a g!a] -> 0"
    )]
    fn test_check_overlap(#[case] input_rule1: &str, #[case] input_rule2: &str) {
        use crate::{id::VarId, rule::RuleKind};
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
        use crate::{id::VarId, rule::RuleKind};

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

    #[test]
    fn test_make_critical_pair_set2() {
        let critical_pairs = make_critical_pair_set(&rules());
        critical_pairs.iter().for_each(|cp| {
            println!("{}", cp);
        });
    }

    #[test]
    fn test_rules() {
        let rules = rules();
        rules.iter().for_each(|r| {
            println!("{}", r);
        });
    }

    /// r1: 0+x -> x
    /// r2: (-x)+x -> 0
    /// r3: (x+y)+z -> x+(y+z)
    fn rules() -> Vec<Rule> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let input_rule1 = "x: Int | plus![0 x] -> x";
        let input_rule2 = "x: Int | plus![minus!x x] -> 0";
        let input_rule3 = "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]";
        let rule1 = rule_parser(&types, &ctxts, &opers)
            .parse(input_rule1)
            .unwrap()
            .0;
        let rule2 = rule_parser(&types, &ctxts, &opers)
            .parse(input_rule2)
            .unwrap()
            .0;
        let rule3 = rule_parser(&types, &ctxts, &opers)
            .parse(input_rule3)
            .unwrap()
            .0;
        vec![rule1, rule2, rule3]
    }
}
