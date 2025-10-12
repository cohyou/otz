use std::{collections::BinaryHeap, rc::Rc};

use crate::{
    analyse::analyse,
    context_table::CtxtTable,
    critical_pairs::{find_critical_pairs, make_critical_pair_set, CriticalPair},
    equation::Equation,
    rule::Rule,
    term::Term,
    util::{dispv, eq, opers, types},
};

pub fn complete2(mut eqs: BinaryHeap<Equation>, max: usize) -> Vec<Rule> {
    let mut step = 0;
    let mut next_eq_id = eqs.len() + 1;
    let mut rules = vec![];

    while !eqs.is_empty() && (step < max || max == 0) {
        println!("----step{}----", step);
        disp_eq("eqs:", &eqs);
        dispv("rules:", &rules);

        // 1 orient
        // 1-1 順序つかないものを消す
        let mut eqs_vec = Vec::from(eqs.clone());
        eqs_vec.retain(|eq| {
            analyse(eq.context.clone(), eq.names.clone(), eq.left.clone(), eq.right.clone()).is_some()
        });
        eqs = BinaryHeap::from(eqs_vec);
        disp_eq("1-1 self.eqs:", &eqs);

        // // 1-2 ひとつとりだす（変数も含めた項数が一番少ないもの）
        if let Some(new_pair) = eqs.pop() {
            println!("1-2 new_pair: {}", &new_pair);

            // 2 compose
            // 2-1 ルールそれぞれの後件を正規化する（新ルールを追加したルール群で）
            let new_rule = analyse(
                new_pair.context.clone(),
                new_pair.names.clone(),
                new_pair.left,
                new_pair.right,
            )
            .unwrap();

            next_eq_id += 1;

            // 3 deduct
            // 3-1 新ルールと他のルールに対して危険対作成（自分自身とも
            let critical_pairs1 = rules.iter()
                .map(|rule| find_critical_pairs(&new_rule, rule))
                // .inspect(|cps| dispv("find_cps:", cps))
                ;
            let critical_pair_self = new_rule.find_critical_pairs_with_self();
            // .iter()
            // .map(|cp| cp.refresh_vars());
            let mut new_pairs_3_1 = critical_pairs1.flatten().collect::<Vec<_>>();
            new_pairs_3_1.extend(critical_pair_self);
            let new_pairs_3_1 = new_pairs_3_1
                .iter()
                .map(|cp| cp.refresh_vars())
                .collect::<Vec<_>>();
            dispv("3-1 critical pairs:", &new_pairs_3_1);

            // 3-2 両辺が同じか、入れ替えただけのペアを消す
            dispv("3-2 new_pairs before:", &new_pairs_3_1);
            let new_pairs_3_2 = new_pairs_3_1
                .iter()
                .enumerate()
                .filter(|(i, pair1)| {
                    new_pairs_3_1.iter().enumerate().all(|(j, pair2)| {
                        if i < &j {
                            let pair1_p = pair1.p_term().refresh_vars();
                            let pair1_q = pair1.q_term().refresh_vars();
                            let pair2_p = pair2.p_term().refresh_vars();
                            let pair2_q = pair2.q_term().refresh_vars();

                            !((pair1_p.inner == pair2_q.inner && pair1_q.inner == pair2_p.inner)
                                || (pair1_p.inner == pair2_p.inner
                                    && pair1_q.inner == pair2_q.inner))
                        } else {
                            true
                        }
                    })
                })
                .map(|(_, p)| p.clone())
                .collect::<Vec<_>>();
            dispv("3-2 new_pairs after:", &new_pairs_3_2);

            // 3-3 等式に合体
            let new_eqs = new_pairs_3_2
                .iter()
                .map(|pair| {
                    // let res = Equation(next_eq_id, pair.2.clone(), pair.3.clone(), (pair.0, pair.1));
                    let res = Equation {
                        context: pair.context.clone(),
                        names: pair.names.clone(),
                        left: pair.p.clone(),
                        right: pair.q.clone(),
                    };
                    next_eq_id += 1;
                    res
                })
                .collect::<Vec<_>>();
            eqs.extend(new_eqs);
            disp_eq("3-3 self.eqs after:", &eqs);

            // 4 collapse
            // 4-1 ルールそれぞれが新ルールに当てはまるか確かめて、当てはまるものは消す
            dispv("4-1 rules before:", &rules);
            // rules.retain(|r| reduxes_in(r.s.clone(), &new_rule).is_empty() );
            rules.retain(|rule| rule.before().find_redexes_from(&new_rule).is_empty());
            dispv("4-1 rules after:", &rules);

            // 5 join
            // 5-1 ルールに新ルール追加
            rules.push(new_rule);
            disp_rl2("5-1 rules:", &rules);

            // 6 simplify
            // 6-1 新ルールリストで等式内をすべて正規化
            disp_eq("6-1 eqs before:", &eqs);
            let mut new_eqs_6 = eqs
                .iter()
                .map(|eq| Equation {
                    context: eq.context.clone(),
                    names: eq.names.clone(),
                    left: eq.left_term().normalize2(&rules).inner.clone(),
                    right: eq.right_term().normalize2(&rules).inner.clone(),
                })
                .collect::<Vec<_>>();
            dispv("6-1 eqs after:", &new_eqs_6);

            // 7 delete
            // 7-1 等式のうち、両辺が同じものを消す
            new_eqs_6.retain(|eq| eq.left != eq.right);
            dispv("7-1 eqs:", &new_eqs_6);

            //             // 両辺を入れ替えただけのものが等式内にあれば消す
            //             let new_eqs_7 = new_eqs_6.clone();
            //             let new_pairs_7_2 = new_eqs_7.iter().enumerate().filter(|(i, pair1)| {
            //                 new_eqs_6.iter().enumerate().all(|(j, pair2)| {
            //                     if i < &j {
            //                         let pair1_p = pair1.left_term().refresh_vars();
            //                         let pair1_q = pair1.right_term().refresh_vars();
            //                         let pair2_p = pair2.left_term().refresh_vars();
            //                         let pair2_q = pair2.right_term().refresh_vars();

            //                         !(
            //                             (pair1_p.inner == pair2_q.inner && pair1_q.inner == pair2_p.inner) ||
            //                             (pair1_p.inner == pair2_p.inner && pair1_q.inner == pair2_q.inner)
            //                         )
            //                     } else {
            //                         true
            //                     }
            //                 })
            //             }).map(|(_,p)|p.clone()).collect::<Vec<_>>();
            // dispv("7-2 eqs:", &new_pairs_7_2);
            // eqs = BinaryHeap::from(new_pairs_7_2);

            eqs = BinaryHeap::from(new_eqs_6);
        }

        step += 1;
    }
    dbg!(step);
    rules
}

pub fn disp_eq(title: &str, eqs: &BinaryHeap<Equation>) {
    println!("{} [", title);
    eqs.iter().for_each(|item| {
        println!("    {}", item);
    });
    println!("]");
}

#[allow(unused)]
pub fn disp_rl2(title: &str, rules: &Vec<Rule>) {
    println!("{} [", title);
    rules.iter().for_each(|item| {
        println!("    {}", item);
    });
    println!("]");
}

pub fn complete(eqs: &Vec<Equation>, limit: usize) -> BinaryHeap<Rule> {
    let mut rules = eqs
        .iter()
        .filter_map(|eq| analyse(eq.context.clone(), eq.names.clone(), eq.left.clone(), eq.right.clone()))
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

        println!("NUMBER: {}", i);
        println!("before: ");
        dispv_cp(&critical_pairs);
        critical_pairs = critical_pairs
            .iter()
            .enumerate()
            .filter(|(i, pair1)| {
                critical_pairs.iter().enumerate().all(|(j, pair2)| {
                    if i < &j {
                        let pair1_p = pair1.p_term().normalize(&rules);
                        let pair1_q = pair1.q_term().normalize(&rules);
                        let pair2_p = pair2.p_term().normalize(&rules);
                        let pair2_q = pair2.q_term().normalize(&rules);

                        !((pair1_p == pair2_q && pair1_q == pair2_p)
                            || (pair1_p == pair2_p && pair1_q == pair2_q))
                    } else {
                        true
                    }
                })
            })
            .map(|(_, p)| p.clone())
            .collect::<BinaryHeap<_>>();

        println!("after : ");
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
                    normal_p.inner.clone(),
                    normal_q.inner.clone(),
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
    let opers = opers(vec!["o", "p", "m"]);
    let ctxts = CtxtTable::new();

    let input_rule1 = "x: Int | p![o; x] = x";
    let input_rule2 = "x: Int | p![m!x x] = o;";
    let input_rule3 = "x y z: Int | p![p![x y] z] = p![x p![y z]]";
    let eq1 = eq(input_rule1, &types, &opers, &ctxts);
    let eq2 = eq(input_rule2, &types, &opers, &ctxts);
    let eq3 = eq(input_rule3, &types, &opers, &ctxts);
    vec![eq1, eq2, eq3]
}

#[cfg(test)]
mod tests {
    use crate::completion::{complete, complete2, eqs};

    #[test]
    fn test_complete2() {
        let eqs_bh = std::collections::BinaryHeap::from(eqs());
        let _rules = complete2(eqs_bh, 4);
    }

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
