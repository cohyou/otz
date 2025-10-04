use std::collections::BinaryHeap;

use crate::{analyse::analyse, critical_pairs::find_critical_pairs, equation::Equation, rule::Rule, util::dispv};

pub fn complete3(eqs: Vec<Equation>, limit: usize) -> Vec<Rule> {
    let mut step = 1;
    let mut eqs = BinaryHeap::from(eqs);
    let mut rules = vec![];

    while !eqs.is_empty() && (limit == 0 || step <= limit) {
        rules = complete_inner(step, &mut eqs, &mut rules);
        disp_eq(&eqs);
        eqs = eqs.iter().map(|eq| eq.refresh_vars()).collect();
        let eqs = dedup_eqs(&eqs);
        let eqs = dedup_eqs2(&eqs);
        disp_eq(&eqs);
        dispv("rules: ", &rules);
        if rules.len() == 15 { panic!(); }
        step += 1;
        println!();
    }

    rules
}

fn complete_inner(step: usize, eqs: &mut BinaryHeap<Equation>, rules: &Vec<Rule>) -> Vec<Rule> {
    let mut rules = rules.clone();

    dbg!(step);
    let eq = eqs.pop().unwrap();
    println!("poped: {}", &eq);

    let left = eq.left_term().normalize2(&rules);
    let right = eq.right_term().normalize2(&rules);
    println!("left: {} right: {}", &left, &right);

    (left != right).then(|| {
        let new_rule = analyse(eq.context.clone(), eq.names.clone(), &left.inner, &right.inner).unwrap();
        println!("new_rule: {}", new_rule);

        // α→βと既存rules内のrule毎の危険対の集合を作る
        let cps = rules.iter()
            .flat_map(|rule| find_critical_pairs(&new_rule, rule));
        // 新rule同士での危険対の有無を調べる
        let cps_self = new_rule.find_critical_pairs_with_self();
        let new_cps = cps.chain(cps_self).map(|cp| cp.refresh_vars()).collect::<Vec<_>>();
        dispv("new_cps:", &new_cps);
        
        rules.push(new_rule);

        // 既約にするために、各規則を正規化する
        rules = rules.iter().map(|rule| {
            let self_excluded_rules = rules.iter().cloned().filter(|r| r != rule).collect::<Vec<_>>();
            Rule {
                id: rule.id,
                context: rule.context.clone(),
                names: rule.names.clone(),
                before: rule.before().normalize2(&self_excluded_rules).inner.clone(),
                after: rule.after().normalize2(&rules).inner.clone(),
            }
        }).collect::<Vec<_>>();

        rules.retain(|r| r.before != r.after);

        let new_eqs = new_cps.iter().map(|cp| 
            Equation { context: cp.context.clone(), names: cp.names.clone(), left: cp.p.clone(), right: cp.q.clone() })
            .collect::<Vec<_>>();
        dispv("new_eqs before:", &new_eqs);

        eqs.extend(new_eqs);
    });

    rules
}

fn dedup_eqs(eqs: &BinaryHeap<Equation>) -> BinaryHeap<Equation> {
    let eqs_cloned = eqs.clone();
    let new_eqs = eqs.iter().enumerate().filter(|(i, eq1)| {
        eqs_cloned.iter().enumerate().all(|(j, eq2)| {
            if i < &j {
                !((eq1.left == eq2.left && eq1.right == eq2.right) || 
                  (eq1.left == eq2.right && eq1.right == eq2.left))
            } else {
                true
            }
        })
    }).map(|(_, eq)| eq.clone()).collect::<Vec<_>>();
    BinaryHeap::from(new_eqs)
}

fn dedup_eqs2(eqs: &BinaryHeap<Equation>) -> BinaryHeap<Equation> {
    let eqs_cloned = eqs.clone();
    let new_eqs = eqs.iter().enumerate().filter(|(i, eq1)| {
        eqs_cloned.iter().enumerate().all(|(j, eq2)| {
            if i < &j {
                let eq2_reversed = Equation {
                    context: eq2.context.clone(),
                    names: eq2.names.clone(),
                    left: eq2.right.clone(),
                    right: eq2.left.clone(),
                }.refresh_vars();
                !(eq1.left == eq2_reversed.left && eq1.right == eq2_reversed.right)
            } else {
                true
            }
        })
    }).map(|(_, eq)| eq.clone()).collect::<Vec<_>>();
    BinaryHeap::from(new_eqs)
}

pub fn disp_eq(vec: &BinaryHeap<Equation>) {
    println!("{} eqs:[", vec.len());
    vec.iter().for_each(|item| {
        println!("    {}", item);
    });
    println!("]");
}

#[cfg(test)]
pub mod tests {
    use crate::{
        comp3::complete3,
        context_table::CtxtTable,
        equation::Equation,
        util::{eq, opers, types},
    };

    #[test]
    fn test_complete3() {
        // 59で10rules
        let _rules = complete3(eqs(), 59);
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
}
