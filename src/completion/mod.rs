pub mod analyse;
pub mod critical_pairs;
pub mod overlap;
pub mod renumber;
pub mod rule;
pub mod subst;
pub mod unify;

use std::collections::BinaryHeap;

use crate::{
    completion::analyse::analyse, completion::critical_pairs::{find_critical_pairs, CriticalPair}, equation::Equation, completion::rule::Rule,
};

#[allow(unused)]
use crate::util::dispv;

/// 補完を打ち切る上限。Knuth–Bendix 補完は一般に停止するとは限らない（停止性は決定不能）ので、
/// 上限を超えたら補完を諦めて CompletionError を返す。
///
/// 上限はどれも決定的（実行時間や環境によらない）なので、同じ入力なら必ず同じ結果になる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletionLimits {
    /// 処理する等式の数（ステップ数）の上限
    pub max_steps: usize,
    /// 規則の数の上限
    pub max_rules: usize,
    /// 規則の項の大きさ（関数記号の数）の上限を、入力の等式の項の最大の大きさの何倍にするか。
    /// 記号が有限なら、規則が無限に増え続けるとき項の大きさも際限なく大きくなる。
    /// そのため、停止しない補完の多くはこの上限で早めに打ち切られる。
    pub max_size_ratio: usize,
}

impl Default for CompletionLimits {
    /// サンプルのインスタンス（入力 34 本、45 ステップ、規則 44 本、項の大きさは入力 4 → 規則 5）や
    /// 群の公理（入力 3 本、規則 10 本、項の大きさは入力 2 → 規則 3）には十分な余裕がある値。
    /// max_steps と max_rules は最後の安全装置で、通常は max_size_ratio が先に効く。
    fn default() -> Self {
        CompletionLimits { max_steps: 5_000, max_rules: 1_000, max_size_ratio: 3 }
    }
}

impl CompletionLimits {
    /// 入力の等式から、規則の項の大きさの上限を決める。
    /// 入力がごく小さい場合でも余裕を持たせるため、入力の大きさは 2 以上とみなす。
    fn max_rule_size(&self, eqs: &[Equation]) -> usize {
        let max_input_size = eqs
            .iter()
            .map(|eq| eq.left.size().max(eq.right.size()))
            .max()
            .unwrap_or(0)
            .max(2);
        self.max_size_ratio * max_input_size
    }
}

/// 補完が上限までに終わらなかったことを表すエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionError {
    /// ステップ数が上限に達した
    StepLimitExceeded { max_steps: usize, rules: usize },
    /// 規則の数が上限を超えた
    RuleLimitExceeded { max_rules: usize, steps: usize },
    /// 規則の項の大きさが上限を超えた
    RuleSizeExceeded { max_size: usize, steps: usize },
}

impl std::fmt::Display for CompletionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionError::StepLimitExceeded { max_steps, rules } => write!(
                f,
                "completion did not terminate within {} steps ({} rules so far)",
                max_steps, rules
            ),
            CompletionError::RuleLimitExceeded { max_rules, steps } => write!(
                f,
                "completion produced more than {} rules (after {} steps)",
                max_rules, steps
            ),
            CompletionError::RuleSizeExceeded { max_size, steps } => write!(
                f,
                "completion produced a rule larger than size {} (after {} steps)",
                max_size, steps
            ),
        }
    }
}

impl std::error::Error for CompletionError {}

/// Knuth–Bendix 補完。等式の集合から、合流性と停止性を持つ書換え規則の集合を作る。
/// 上限（limits）までに終わらなければ CompletionError を返す。
/// 向き付けできない等式（可換律など）は保留し、規則にしない（健全だが不完全になり得る）。
pub fn complete(eqs: Vec<Equation>, limits: CompletionLimits) -> Result<Vec<Rule>, CompletionError> {
    let max_rule_size = limits.max_rule_size(&eqs);
    let mut step = 1;
    let mut eqs = BinaryHeap::from(eqs);
    let mut rules = vec![];
    // 向き付けできなかった（LPOで比較不能な）等式。規則が増えたら再挑戦する
    let mut postponed: Vec<Equation> = vec![];

    while !eqs.is_empty() {
        if step > limits.max_steps {
            return Err(CompletionError::StepLimitExceeded { max_steps: limits.max_steps, rules: rules.len() });
        }
        // dbg!(step);
        let eq = eqs.pop().unwrap();
        // println!("POPED: {}", &eq);
        let (new_eqs, new_rules) = match complete_inner(step, &eq, &mut rules) {
            Some(result) => result,
            None => {
                postponed.push(eq);
                step += 1;
                continue;
            }
        };
        if new_rules != rules {
            eqs.extend(postponed.drain(..));
        }

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

        // if rules.len() != new_rules.len() {
        //     dispv("rules:", &rules);
        // }
        rules = new_rules;

        if rules.len() > limits.max_rules {
            return Err(CompletionError::RuleLimitExceeded { max_rules: limits.max_rules, steps: step });
        }
        if rules.iter().any(|r| r.before.size().max(r.after.size()) > max_rule_size) {
            return Err(CompletionError::RuleSizeExceeded { max_size: max_rule_size, steps: step });
        }
        step += 1;
        // println!();
    }

    Ok(rules)
}

/// 向き付けできない等式の場合は None を返す
fn complete_inner(_step: usize, eq: &Equation, rules: &Vec<Rule>) -> Option<(Vec<Equation>, Vec<Rule>)> {
    let mut rules = rules.clone();

    let left = eq.left_term().normalize(&rules);
    let right = eq.right_term().normalize(&rules);
    // println!("left: {}  | right: {}", &left, &right);

    if left == right {
        return Some((vec![], rules));
    }
    let new_rule = analyse(eq.context.clone(), eq.names.clone(), left.inner.clone(), right.inner.clone())?;
    let new_eqs = {
        // println!("new_rule: {}", new_rule);

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

        let new_eqs = new_cps.iter().map(|cp| 
            Equation { context: cp.context.clone(), names: cp.names.clone(), left: cp.p.clone(), right: cp.q.clone() })
            .collect::<Vec<_>>();
        // dispv("new_eqs before:", &new_eqs);

        // eqs.extend(new_eqs);
        new_eqs
    };

    Some((new_eqs, rules))
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
        completion::{complete, CompletionError, CompletionLimits},
        context_table::CtxtTable,
        equation::Equation,
        util::{dispv, eq, opers, types},
    };

    /// 群の公理を補完すると、よく知られた10本の完備な書換え系になる
    #[test]
    fn test_complete() {
        let input = complete_eqs();
        let rules = complete(input.clone(), CompletionLimits::default()).unwrap();
        dispv("FINAL RULES:", &rules);
        assert_eq!(rules.len(), 10);
        assert_all_joinable(&input, &rules);
    }

    /// Knuth–Bendix の古典例: 群の3公理だけから同じ10本の規則が得られる
    #[test]
    fn test_complete_group_from_three_axioms() {
        let input = super::eqs();
        let rules = complete(input.clone(), CompletionLimits::default()).unwrap();
        dispv("FINAL RULES:", &rules);
        assert_eq!(rules.len(), 10);
        assert_all_joinable(&input, &rules);
        assert_all_joinable(&complete_eqs(), &rules);
    }

    /// 補完が停止しない等式は、項の大きさの上限で早めに打ち切られる（#16）。
    /// f(g(f(x))) = g(f(x)) からは f(g^n(f(x))) → g^n(f(x)) のような規則が無限に生じる。
    /// 演算子の優先順位（宣言順）によらず停止しないことも確かめる。
    #[rstest::rstest]
    #[case(vec!["f", "g"], "x: Int | f!g!f!x = g!f!x")]
    #[case(vec!["g", "f"], "x: Int | f!g!f!x = g!f!x")]
    #[case(vec!["f", "g"], "x: Int | f!g!f!x = f!g!x")]
    fn test_complete_diverging(#[case] decl: Vec<&str>, #[case] input: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(decl);
        let ctxts = CtxtTable::new();
        let input = vec![eq(input, &types, &opers, &ctxts)];

        let result = complete(input, CompletionLimits::default());
        // 入力の項の大きさは 3 なので、上限は 3 × 3 = 9
        assert!(
            matches!(result, Err(CompletionError::RuleSizeExceeded { max_size: 9, .. })),
            "{:?}",
            result
        );
    }

    /// ステップ数・規則の数の上限も効く。どちらも決定的なので、同じ入力なら同じエラーになる
    #[test]
    fn test_complete_step_and_rule_limits() {
        let steps = CompletionLimits { max_steps: 5, ..CompletionLimits::default() };
        assert_eq!(
            complete(super::eqs(), steps),
            Err(CompletionError::StepLimitExceeded { max_steps: 5, rules: 5 }),
        );

        let rules = CompletionLimits { max_rules: 3, ..CompletionLimits::default() };
        assert!(matches!(
            complete(super::eqs(), rules),
            Err(CompletionError::RuleLimitExceeded { max_rules: 3, .. })
        ));
    }

    fn assert_all_joinable(eqs: &[Equation], rules: &Vec<crate::completion::rule::Rule>) {
        for eq in eqs {
            let left = eq.left_term().normalize(rules);
            let right = eq.right_term().normalize(rules);
            assert_eq!(left, right, "not joinable: {}", eq);
        }
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
                           "x y: Int | m!p![x y] = p![m!y m!x]", // -(x + y) = -y + -x
                        ];
        input_rules.iter()
            .map(|r| eq(r, &types, &opers, &ctxts)).collect()
    }
}

