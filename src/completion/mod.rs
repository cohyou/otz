pub mod analyse;
pub mod critical_pairs;
pub mod overlap;
pub mod renumber;
pub mod rule;
pub mod subst;
pub mod unify;

use std::collections::BinaryHeap;

use crate::{completion::analyse::analyse, completion::critical_pairs::{find_critical_pairs, CriticalPair}, equation::Equation, completion::rule::Rule, util::dispv};

pub fn complete(eqs: Vec<Equation>, limit: usize) -> Vec<Rule> {
    let mut step = 1;
    let mut eqs = BinaryHeap::from(eqs);
    let mut rules = vec![];

    while !eqs.is_empty() && (limit == 0 || step <= limit) {
        dbg!(step);
        let eq = eqs.pop().unwrap();
        println!("POPED: {}", &eq);
        let (new_eqs, new_rules) = complete_inner(step, &eq, &mut rules);

        eqs.extend(new_eqs);
        // disp_eq(&eqs);
        let refreshed_eqs = eqs.iter().map(|eq| eq.refresh_vars()).collect();
        let deduped_eqs = dedup_eqs(&refreshed_eqs);
        let new_eqs = dedup_eqs2(&deduped_eqs);
        if eqs.len() != new_eqs.len() {
            // disp_eq(&eqs);
            // disp_eq(&new_eqs);
            eqs = new_eqs;
        }

        if rules.len() != new_rules.len() {
            dispv("rules:", &rules);
        }
        rules = new_rules;
        
        if rules.len() == 100 { panic!(); }
        step += 1;
        // println!();
    }

    rules
}

fn complete_inner(_step: usize, eq: &Equation, rules: &Vec<Rule>) -> (Vec<Equation>, Vec<Rule>) {
    let mut rules = rules.clone();

    let left = eq.left_term().normalize(&rules);
    let right = eq.right_term().normalize(&rules);
    // println!("left: {}  | right: {}", &left, &right);

    let mut new_eqs = vec![];
    (left != right).then(|| {
        let new_rule = analyse(eq.context.clone(), eq.names.clone(), left.inner.clone(), right.inner.clone()).unwrap();
        println!("new_rule: {}", new_rule);

        // α→βと既存rules内のrule毎の危険対の集合を作る
        let cps = rules.iter()
            .flat_map(|rule| find_critical_pairs(&new_rule, rule));
        // 新rule同士での危険対の有無を調べる
        let cps_self = new_rule.find_critical_pairs_with_self();



        let mut new_cps = cps.chain(cps_self).map(|cp| cp.refresh_vars())
        .map(|cp| {
            let p = cp.p_term().normalize(&rules).inner.clone();
            let q = cp.q_term().normalize(&rules).inner.clone();
            CriticalPair { context: cp.context.clone(), names: cp.names.clone(), p, q }
        })
        .collect::<Vec<_>>();
        // dispv("new_cps:", &new_cps);
        new_cps.retain(|cp| cp.p != cp.q);

        rules.push(new_rule);
        
        // dispv("BEFORE RULES", &rules);
        // 既約にするために、各規則を正規化する
        rules = rules.iter().map(|rule| {
            let self_excluded_rules = rules.iter().cloned().filter(|r| r != rule).collect::<Vec<_>>();
            Rule {
                id: rule.id,
                context: rule.context.clone(),
                names: rule.names.clone(),
                before: rule.before().normalize(&self_excluded_rules).inner.clone(),
                after: rule.after().normalize(&rules).inner.clone(),
            }
        }).collect::<Vec<_>>();
        // dispv("AFTER RULES", &rules);
        rules.retain(|r| r.before != r.after);

        new_eqs = new_cps.iter().map(|cp| 
            Equation { context: cp.context.clone(), names: cp.names.clone(), left: cp.p.clone(), right: cp.q.clone() })
            .collect::<Vec<_>>();
        // dispv("new_eqs before:", &new_eqs);

        // eqs.extend(new_eqs);
    });

    (new_eqs, rules)
}

fn dedup_eqs(eqs: &BinaryHeap<Equation>) -> BinaryHeap<Equation> {
    let eqs_cloned = eqs.clone();
    let new_eqs = eqs.iter().enumerate().filter(|(i, eq1)| {
        eqs_cloned.iter().enumerate().all(|(j, eq2)| {
            if i < &j {
                // if eq1.left == eq2.left && eq1.right == eq2.right {
                //     println!("dedup_eqs: ({}){} and ({}){} are duplicates", i, eq1, j, eq2);
                // }
                // if eq1.left == eq2.right && eq1.right == eq2.left {
                //     println!("dedup_eqs: ({}){} and ({}){} are duplicates (by reversal)", i, eq1, j, eq2);
                // }
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
                // if eq1.left == eq2_reversed.left && eq1.right == eq2_reversed.right {
                //     println!("dedup_eqs2: ({}){} and ({}){} are duplicates (by reversal)", i, eq1, j, eq2);
                // }
                !(eq1.left == eq2_reversed.left && eq1.right == eq2_reversed.right)
            } else {
                true
            }
        })
    }).map(|(_, eq)| eq.clone()).collect::<Vec<_>>();
    BinaryHeap::from(new_eqs)
}

#[allow(dead_code)]
pub fn disp_eq(vec: &BinaryHeap<Equation>) {
    println!("{} eqs:[", vec.len());
    vec.iter().for_each(|item| {
        println!("    {}", item);
    });
    println!("]");
}

#[allow(dead_code)]
pub fn eqs() -> Vec<Equation> {
    use crate::{
        context_table::CtxtTable,
        util::{eq, opers, types},
    };

    let types = types(vec!["Int"]);
    let opers = opers(vec!["o", "p", "m"]);
    let ctxts = CtxtTable::new();

    let input_rules = ["x: Int | p![o; x] = x",
                        "x: Int | p![m!x x] = o;",
                        "x y z: Int | p![p![x y] z] = p![x p![y z]]"];
    input_rules.iter()
        .map(|r| eq(r, &types, &opers, &ctxts)).collect()
}

#[cfg(test)]
pub mod tests {
    use crate::{
        completion::complete,
        context_table::CtxtTable,
        equation::Equation,
        util::{dispv, eq, opers, types},
    };

    #[test]
    fn test_complete() {
        // 59で10rules
        let rules = complete(complete_eqs(),21);
        dispv("FINAL RULES:", &rules);
    }

    pub fn complete_eqs() -> Vec<Equation> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["o", "p", "m"]);
        let ctxts = CtxtTable::new();

        let input_rules = ["x: Int | p![o; x] = x",
                           "x: Int | p![m!x x] = o;",
                           "x y z: Int | p![p![x y] z] = p![x p![y z]]",
                           "x z: Int | p![m!x p![x z]] = z", // -x + (x + z) = z
                           "x: Int | p![x m!x] = o;", // x + -x = 0
                           "x: Int | p![x o;] = x", // x + 0 = x
                           "x y: Int | p![y p![m!y x]] = x", // y + (-y + x) = x
                           "| m!o; = o;", // -0 = 0
                           "x: Int | m!m!x = x", // --x = x
                           "x y: Int | m!p![x y] = p![m!x m!y]", // -(x + y) = -y + -x
                        ];
        input_rules.iter()
            .map(|r| eq(r, &types, &opers, &ctxts)).collect()
    }
}
