use std::{collections::HashMap, rc::Rc};

use crate::{
    analyse::analyse,
    context::Context,
    equation::Equation,
    id::VarId,
    overlap::Overlap,
    rule::{Rule, RuleKind},
    subst::{Subst, Var},
    symbol_table::Names,
    term::{Term, TermInner},
};

pub fn complete(eqs: &Vec<Equation>) -> Vec<Rule> {
    let mut rules = eqs
        .iter()
        .map(|eq| analyse(eq.context.clone(), eq.names.clone(), &eq.left, &eq.right))
        .collect::<Vec<_>>();
    let mut critical_pairs = make_critical_pair_set(&rules);

    println!("--- {} CPs [", critical_pairs.len());
    critical_pairs.iter().for_each(|r| {
        println!("{}", r);
    });
    println!("] ---");

    while !critical_pairs.is_empty() {
        let cp = critical_pairs.pop().unwrap();
        // p, qのrulesに関しての正規形p^,q^を求める
        let normal_p = cp.p_term().normalize(&rules);
        let normal_q = cp.q_term().normalize(&rules);

        if normal_p != normal_q {
            println!("NOT EQUAL ({} != {})", normal_p, normal_q);

            let new_rule = analyse(cp.context, cp.names, &normal_p.inner, &normal_q.inner);
            rules.push(new_rule.clone());
            // α→βと既存rules内のrule毎の危険対の集合を作る
            let new_pairs = rules
                .iter()
                .inspect(|r| print!("find new CP <| {} and {} makes ", &new_rule, r))
                .flat_map(|rule| find_critical_pairs(&new_rule, rule))
                .inspect(|cp| println!("{} |>", cp))
                .collect::<Vec<_>>();
            critical_pairs.extend(new_pairs);
        } else {
            println!("EQUAL ({} == {})", normal_p, normal_q);
        }

        println!("--- {} CPs [", critical_pairs.len());
        critical_pairs.iter().for_each(|r| {
            println!("{}", r);
        });
        println!("] ---");
    }
    rules
}

pub struct CriticalPair {
    pub context: Context,
    pub names: Names,
    pub p: Rc<TermInner>,
    pub q: Rc<TermInner>,
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

impl Overlap {
    /// r1:s1->t1 r2:s2->t2
    /// r2がuでmguθによりr1に重なるとする
    /// その場合<θs1[u<-t2], θt1>をr1とr2の危険対という
    /// ちなみに、一般性を失わず、s1とs2の変数は重ならないとしてよい
    pub fn to_critical_pair(&self) -> Option<CriticalPair> {
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
mod tests {
    use combine::Parser;

    use crate::{
        completion::{complete, make_critical_pair_set},
        context_table::CtxtTable,
        equation::Equation,
        parser::{equation::equation_parser, rule::rule_parser},
        rule::{Rule, RuleKind},
        util::{opers, types},
    };

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

    #[test]
    fn test_make_critical_pair_set2() {
        let critical_pairs = make_critical_pair_set(&rules());
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
