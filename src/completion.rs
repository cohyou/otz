use std::{collections::BinaryHeap, rc::Rc};

use crate::{
    analyse::analyse,
    context_table::CtxtTable,
    critical_pairs::{find_critical_pairs, make_critical_pair_set, CriticalPair},
    equation::Equation,
    rule::Rule,
    term::Term,
    util::{eq, opers, types},
};

pub fn complete(eqs: &Vec<Equation>, limit: usize) -> BinaryHeap<Rule> {
    let mut rules = eqs
        .iter()
        .filter_map(|eq| analyse(eq.context.clone(), eq.names.clone(), &eq.left, &eq.right))
        .collect::<BinaryHeap<_>>();
    let critical_pairs = make_critical_pair_set(&rules)
        .iter()
        .map(|cp| cp.refresh_vars())
        .collect::<Vec<_>>();
    // dispv(&critical_pairs);

    let mut critical_pairs = BinaryHeap::from(critical_pairs);
    let mut i = 1;
    while !critical_pairs.is_empty() {
        let cp = critical_pairs.pop().unwrap();

        cp.to_rule(&rules).map(|new_rule| {
            // α→βと既存rules内のrule毎の危険対の集合を作る
            let new_pairs = rules
                .iter()
                .flat_map(|rule| find_critical_pairs(&new_rule, rule))
                .collect::<Vec<_>>();
            extend_cps(&mut critical_pairs, new_pairs, &rules);

            // 新rule同士での危険対の有無を調べる
            let new_pairs_self = new_rule
                .find_critical_pairs_with_self()
                .iter()
                .map(|cp| cp.refresh_vars())
                .collect::<Vec<_>>();
            extend_cps(&mut critical_pairs, new_pairs_self, &rules);

            rules.push(new_rule.clone());
        });

        critical_pairs = critical_pairs.iter().enumerate().filter(|(i, pair1)| {
            critical_pairs.iter().enumerate().all(|(j, pair2)| {
                if i < &j {
                    !(pair1.p == pair2.q && pair1.q == pair2.p || 
                        pair1.p == pair2.p && pair1.q == pair2.q
                    )
                } else {
                    true
                }
            })
        }).map(|(_,p)|p.clone()).collect::<BinaryHeap<_>>();

        println!("NUMBER: {}", i);
        dispv_cp(&critical_pairs);
        disp_rl(&rules);
        i = i + 1;
        if limit > 0 && i >= limit {
            break;
        }
    }
    rules
}

fn extend_cps(
    critical_pairs: &mut BinaryHeap<CriticalPair>,
    new_pairs: Vec<CriticalPair>,
    rules: &BinaryHeap<Rule>,
) {
    new_pairs
        .into_iter()
        .map(|cp| cp.refresh_vars())
        .filter(|cp| cp.make_normalized(&rules).is_some())
        // .filter(|cp|!critical_pairs.contains(cp))
        .for_each(|cp| {
            critical_pairs.push(cp);
        });
}

impl CriticalPair {
    fn to_rule(&self, rules: &BinaryHeap<Rule>) -> Option<Rule> {
        self.make_normalized(rules)
            .map(|(normal_p, normal_q)| {
                analyse(
                    self.context.clone(),
                    self.names.clone(),
                    &normal_p.inner,
                    &normal_q.inner,
                )
                .map(|r| {
                    let new_rule = r.refresh_vars();
                    // println!("NEW RULE: {}", &new_rule);
                    new_rule
                })
            })
            .flatten()
    }

    fn make_normalized(&self, rules: &BinaryHeap<Rule>) -> Option<(Rc<Term>, Rc<Term>)> {
        // p, qのrulesに関しての正規形p^,q^を求める
        let p = self.p_term();
        let q = self.q_term();
        let normal_p = p.normalize(rules);
        let normal_q = q.normalize(rules);
        if normal_p != normal_q {
            // printpq(&p, &q, &normal_p, &normal_q);
            Some((normal_p, normal_q))
        } else {
            // println!("EQUAL ({} == {})", p, q);
            None
        }
    }
}

fn _printpq(p: &Rc<Term>, q: &Rc<Term>, normal_p: &Rc<Term>, normal_q: &Rc<Term>) {
    let left = if p == normal_p {
        format!("{}", p)
    } else {
        format!("<{} => {}>", p, normal_p)
    };
    let right = if q == normal_q {
        format!("{}", q)
    } else {
        format!("<{} => {}>", q, normal_q)
    };
    println!("NOT EQUAL ({} != {})", left, right);
}

pub fn dispv_cp(critical_pairs: &BinaryHeap<CriticalPair>) {
    println!("--- {} CPs [", critical_pairs.len());
    critical_pairs.iter().for_each(|cp| {
        // println!("    {}, < {:?} | {:?} >", cp, cp.p.vars(), cp.q.vars());
        println!("    {}", cp);
    });
    println!("] ---");
}

pub fn disp_rl(vec: &BinaryHeap<Rule>) {
    println!("--- {} rules [", vec.len());
    vec.iter().for_each(|item| {
        println!("    {}", item);
    });
    println!("] ---");
}

pub fn eqs() -> Vec<Equation> {
    let types = types(vec!["Int"]);
    let opers = opers(vec!["plus", "minus"]);
    let ctxts = CtxtTable::new();

    let input_rule1 = "x: Int | plus![0 x] = x";
    let input_rule2 = "x: Int | plus![minus!x x] = 0";
    let input_rule3 = "x: Int y: Int z: Int | plus![plus![x y] z] = plus![x plus![y z]]";
    let eq1 = eq(input_rule1, &types, &opers, &ctxts);
    let eq2 = eq(input_rule2, &types, &opers, &ctxts);
    let eq3 = eq(input_rule3, &types, &opers, &ctxts);
    vec![eq1, eq2, eq3]
}

#[cfg(test)]
mod tests {
    use crate::completion::{complete, eqs};

    #[test]
    fn test_complete() {
        // 9で8ruleできる
        // 14で9個
        // 15 10
        // 16 11
        // 17 12
        // 21 13
        // 27 14
        let _rules = complete(&eqs(), 100);
        // dispv(&rules);
    }
}
