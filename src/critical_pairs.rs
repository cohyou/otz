use std::{
    collections::{BinaryHeap, HashMap},
    rc::Rc,
};

use crate::{
    context::Context,
    id::VarId,
    overlap::Overlap,
    rule::{Rule, RuleKind},
    subst::{Subst, Var},
    symbol_table::Names,
    term::{Term, TermInner},
};

#[derive(PartialEq, Clone)]
pub struct CriticalPair {
    pub context: Rc<Context>,
    pub names: Rc<Names>,
    pub p: Rc<TermInner>,
    pub q: Rc<TermInner>,
}

impl std::cmp::PartialOrd for CriticalPair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (std::cmp::max(other.p.var_size(), other.q.var_size()))
            .partial_cmp(std::cmp::max(&self.p.var_size(), &self.q.var_size()))
    }
}
impl std::cmp::Ord for CriticalPair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::cmp::Eq for CriticalPair {}

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

pub fn prepare_rules(rules: &BinaryHeap<Rule>) -> Vec<Rule> {
    rules
        .iter()
        .enumerate()
        .map(|(idx, rule)| prepare_rule(rule, idx))
        .collect()
}

pub fn prepare_rules2(rules: &Vec<Rule>) -> Vec<Rule> {
    rules
        .iter()
        .enumerate()
        .map(|(idx, rule)| prepare_rule(rule, idx))
        .collect()
}

pub fn prepare_rule(rule: &Rule, idx: usize) -> Rule {
    let mut r = rule.clone();
    r.id = Some(idx);
    r
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

pub fn make_critical_pair_set(rules: &BinaryHeap<Rule>) -> Vec<CriticalPair> {
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
                        // println!("SAME RULE: {} {}", &r1, &r2);
                        r1.check_overlap::<(VarId, usize, RuleKind)>(&r2)
                    } else {
                        let r1 = rule1.make_vars_ruled(RuleKind::NotSet);
                        let r2 = rule2.make_vars_ruled(RuleKind::NotSet);
                        // dbg!(&r1, &r2);
                        r1.check_overlap::<(VarId, usize, RuleKind)>(&r2)
                        // rule1.check_overlap::<(VarId, usize, RuleKind)>(rule2)
                    };
                    // overlaps.iter().for_each(|ol| {
                    //     println!("overlap: {}", ol);
                    // });
                    overlaps
                        .iter()
                        .filter_map(Overlap::to_critical_pair)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// rule2とrule1との互いの危険対を探す
/// rule2とrule2・rule1とrule1の組み合わせは探さない
pub fn find_critical_pairs(rule1: &Rule, rule2: &Rule) -> Vec<CriticalPair> {
    // println!("rule1: {} rule2: {}", rule1, rule2);
    let rule1 = prepare_rule(rule1, 1).make_vars_ruled(RuleKind::NotSet);
    let rule2 = prepare_rule(rule2, 2).make_vars_ruled(RuleKind::NotSet);

    let overlaps1 = rule1.check_overlap::<(VarId, usize, RuleKind)>(&rule2);
    let pairs1 = overlaps1.iter().filter_map(Overlap::to_critical_pair)
        // .inspect(|pair| println!("NEW CP {{\n    {} が\n    {} に重なり\n    {}\n}}", rule2, rule1, pair) )
        ;
    let overlaps2 = rule2.check_overlap::<(VarId, usize, RuleKind)>(&rule1);
    let pairs2 = overlaps2.iter().filter_map(Overlap::to_critical_pair)
        // .inspect(|pair| println!("NEW CP {{\n    {} が\n    {} に重なり\n    {}\n}}", rule1, rule2, pair) )
        ;
    pairs1.chain(pairs2).collect()
}

impl Rule {
    pub fn find_critical_pairs_with_self(&self) -> Vec<CriticalPair> {
        let rules_self = BinaryHeap::from(vec![self.clone()]);
        make_critical_pair_set(&rules_self)
    }
}

impl Overlap {
    /// r1:s1->t1 r2:s2->t2
    /// r2がuでmguθによりr1に重なるとする
    /// その場合<θs1[u<-t2], θt1>をr1とr2の危険対という
    /// ちなみに、一般性を失わず、s1とs2の変数は重ならないとしてよい
    pub fn to_critical_pair(&self) -> Option<CriticalPair> {
        // 重なりを危険対に変換する
        // println!("to_critical_pair: {}", self);

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
mod tests {
    use combine::Parser;
    use rstest::rstest;

    use crate::{
        context_table::CtxtTable,
        critical_pairs::{find_critical_pairs, make_critical_pair_set},
        parser::rule::rule_parser,
        rule::{Rule, RuleKind},
        util::{opers, rl, types},
    };

    #[rstest]
    #[case(
        "x y: Int | plus![minus!x plus![x y]] -> y",
        "x: Int | plus![minus!x x] -> 0"
    )]
    #[case(
        "x z: Int | plus![minus!x plus![x z]] -> z",
        "x: Int | plus![0 x] -> x"
    )]
    fn test_find_critical_pairs(#[case] r1: &str, #[case] r2: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let rule1 = rl(r1, &types, &opers, &ctxts);
        let rule2 = rl(r2, &types, &opers, &ctxts);

        let r = find_critical_pairs(&rule1, &rule2);
        dbg!(r);
    }

    #[rstest]
    #[case("x y z: Int | plus![plus![x y] z] -> plus![x plus![y z]]")]
    #[case("x z: Int | plus![minus!x plus![x z]] -> z")]
    fn test_make_critical_pair_set(#[case] input: &str) {
        use crate::{
            context_table::CtxtTable,
            critical_pairs::make_critical_pair_set,
            parser::rule::rule_parser,
            util::{opers, types},
        };

        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let mut rule1 = rule_parser(&types, &ctxts, &opers)
            .parse(input)
            .clone()
            .unwrap()
            .0;
        rule1.id = Some(1);
        let critical_pairs = make_critical_pair_set(&(vec![rule1].into()));
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

    #[test]
    fn test_make_critical_pair_set2() {
        let critical_pairs = make_critical_pair_set(&(rules().into()));
        critical_pairs.iter().for_each(|cp| {
            println!("{}", cp);
        });
    }

    #[test]
    fn test_display_rules() {
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
