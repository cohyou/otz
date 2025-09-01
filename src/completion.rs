use std::{collections::HashMap, rc::Rc};

use crate::{
    context::Context, equation::Equation, id::VarId, rule::{Rule, RuleKind}, subst::{Subst, Var}, subterm::{Position, Subterm}, term::{Term, TermInner}, unify::unify
};

pub fn complete(eqs: &Vec<Equation>) -> Vec<Rule> {
    let mut rules = eqs
        .iter()
        .map(|eq| analyse(&eq.left, &eq.right))
        .collect::<Vec<_>>();
    let mut critical_pairs = make_critical_pair_set(&rules);

    while !critical_pairs.is_empty() {
        let cp = critical_pairs.pop().unwrap();
        // p, qのrulesに関しての正規形p^,q^を求める
        let normal_p = cp.p_term().normalize(&rules);
        let normal_q = cp.q_term().normalize(&rules);
        if normal_p != normal_q {
            let new_rule = analyse(&normal_p.inner, &normal_q.inner);
            rules.push(new_rule.clone());
            // α→βと既存rules内のrule毎の危険対の集合を作る
            let new_pairs = rules
                .iter()
                .flat_map(|rule| find_critical_pairs(&new_rule, rule))
                .collect::<Vec<_>>();
            critical_pairs.extend(new_pairs);
        }
    }
    rules
}

fn analyse(_left: &Rc<TermInner>, _right: &Rc<TermInner>) -> Rule {
    unimplemented!()
}

struct CriticalPair {
    context: Context,
    p: Rc<TermInner>,
    q: Rc<TermInner>,
}

impl CriticalPair {
    pub fn p_term(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(), 
            inner: self.p.clone(),
        })
    }
    pub fn q_term(&self) -> Rc<Term> {
        Rc::new(Term {
            context: self.context.clone(), 
            inner: self.q.clone(),
        })
    }
}

impl std::fmt::Debug for CriticalPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "< {:?} | {:?} >", self.p, self.q)
    }
}

fn prepare_rules(rules: &Vec<Rule>) -> Vec<Rule> {
    rules.iter().enumerate().map(|(idx, rule)| {
        let mut r = rule.clone();
        r.id = Some(idx);
        r
    }).collect()
}

impl Term {
    pub fn vars_ruled_term(&self, rule_id: usize, kind: RuleKind) -> Rc<TermInner> {
        let subst = HashMap::new();
        let subst = self.context.0.keys().fold(subst, |mut subst, v| {
            let ruled_var = TermInner::RuledVar(v.clone(), rule_id, kind.clone());
            let var = Var::Ruled(v.clone(), rule_id, kind.clone());
            subst.insert(var, Rc::new(ruled_var));
            subst
        });
        self.substitute(&Subst(subst.clone())).inner
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
                    rule1
                        .check_overlap::<(VarId, usize, RuleKind)>(rule2)
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
    fn check_overlap<T:Eq + std::hash::Hash>(&self, from: &Rule) -> Vec<Overlap> {
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
                let s1_is_not_var = !matches!(s1_sub.inner.as_ref(), TermInner::Var(_));
                // dbg!(&s1_sub.inner, s1_is_not_var);
                // r1≡r2ならu≠ε(恒等写像=無意味な置換になってしまう)
                let is_not_identity = !(is_same_rule && subterm.pos.is_empty());
                // dbg!(s1_is_not_var, is_not_identity);
                // 上記を前提とする
                (s1_is_not_var && is_not_identity)
                    .then(|| {
                        let (s, t) = if is_same_rule {
                            (
                                s1_sub.vars_ruled_term(self.id.unwrap(), RuleKind::Set1),
                                s2.vars_ruled_term(from.id.unwrap(), RuleKind::Set2)
                            )
                        } else {
                            (s1_sub.inner.clone(), s2.inner.clone())
                        };
                        unify(s, t).map(|theta| (subterm.pos, theta))
                    })
                    .flatten()
            })
            .map(|(pos, theta)| Overlap {
                context: self.context.clone(),
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
        (left != right).then_some(
            CriticalPair {
                context: self.context.clone(),
                p: left.inner.clone(),
                q: right.inner.clone(),
            }
        )
    }
}

#[cfg(test)]
mod test {
    use combine::Parser;

    use crate::{
        completion::{make_critical_pair_set, Overlap},
        context_table::CtxtTable,
        id::{OperId, TypeId},
        parser::rule::rule_parser,
        symbol_table::SymbolTable,
    };

    use rstest::*;

    #[rstest]
    #[case(
        "x1: Int y1: Int z1: Int | f![f![x1 y1] z1] -> f![x1 f![y1 z1]]",
        "a: Int | f![a g!a] -> 0"
    )]
    fn test_check_overlap(#[case] input_rule1: &str, #[case] input_rule2: &str) {
        use crate::{id::VarId, rule::RuleKind};

        let types = SymbolTable::<TypeId>::new();
        types.assign("Int".to_string());
        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        opers.assign("g".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("x1".to_string());
        ctxts.assign_to_current("y1".to_string());
        ctxts.assign_to_current("z1".to_string());
        ctxts.assign_to_current("a".to_string());
        // dbg!(&ctxts);

        let rule1 = rule_parser(&types, &ctxts, &opers).parse(input_rule1);
        let rule2 = rule_parser(&types, &ctxts, &opers).parse(input_rule2);
        dbg!(&rule1);
        dbg!(&rule2);
        let overlap1 = rule1
            .clone()
            .unwrap()
            .0
            .check_overlap::<(VarId, usize, RuleKind)>(&rule2.clone().unwrap().0);
        let overlap2 = rule2.unwrap().0.check_overlap::<(VarId, usize, RuleKind)>(&rule1.unwrap().0);
        dbg!(&overlap1);
        dbg!(&overlap2);
        let critical_pairs1 = overlap1
            .iter()
            .map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        let critical_pairs2 = overlap2
            .iter()
            .map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        dbg!(critical_pairs1);
        dbg!(critical_pairs2);
    }

    #[rstest]
    // #[case("x1: Int | plus![0 x1] -> x1", "x2: Int | plus![minus!x2 x2] -> 0")]
    // #[case("x1: Int | plus![0 x1] -> x1", "x3: Int y3: Int z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]")]
    // #[case("x3: Int y3: Int z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]", "x2: Int | plus![minus!x2 x2] -> 0")]
    #[case(
        "x3: Int y3: Int z3: Int | plus![plus![x3 y3] z3] -> plus![x3 plus![y3 z3]]",
        "x4: Int y4: Int z4: Int | plus![plus![x4 y4] z4] -> plus![x4 plus![y4 z4]]"
    )]
    fn test_check_overlap2(#[case] input_rule1: &str, #[case] input_rule2: &str) {
        use crate::{id::VarId, rule::RuleKind};

        let types = SymbolTable::<TypeId>::new();
        types.assign("Int".to_string());
        let opers = SymbolTable::<OperId>::new();
        opers.assign("plus".to_string());
        opers.assign("minus".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("x1".to_string());
        ctxts.assign_to_current("x2".to_string());
        ctxts.assign_to_current("x3".to_string());
        ctxts.assign_to_current("y3".to_string());
        ctxts.assign_to_current("z3".to_string());
        ctxts.assign_to_current("x4".to_string());
        ctxts.assign_to_current("y4".to_string());
        ctxts.assign_to_current("z4".to_string());

        let rule1 = rule_parser(&types, &ctxts, &opers).parse(input_rule1);
        let rule2 = rule_parser(&types, &ctxts, &opers).parse(input_rule2);
        dbg!(&rule1);
        dbg!(&rule2);
        let overlap1 = rule1
            .clone()
            .unwrap()
            .0
            .check_overlap::<(VarId, usize, RuleKind)>(&rule2.clone().unwrap().0);
        let overlap2 = rule2.unwrap().0.check_overlap::<(VarId, usize, RuleKind)>(&rule1.unwrap().0);
        dbg!(&overlap1);
        dbg!(&overlap2);
        let critical_pairs1 = overlap1
            .iter()
            .filter_map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        let critical_pairs2 = overlap2
            .iter()
            .filter_map(Overlap::to_critical_pair)
            .collect::<Vec<_>>();
        dbg!(critical_pairs1);
        dbg!(critical_pairs2);
    }

    #[test]
    fn test_complete() {
        let types = SymbolTable::<TypeId>::new();
        types.assign("Int".to_string());
        let opers = SymbolTable::<OperId>::new();
        opers.assign("plus".to_string());
        opers.assign("minus".to_string());
        let ctxts12 = CtxtTable::new();
        ctxts12.assign_to_current("x".to_string());
        let ctxts3 = CtxtTable::new();
        ctxts3.assign_to_current("x".to_string());
        ctxts3.assign_to_current("y".to_string());
        ctxts3.assign_to_current("z".to_string());
        // r1: 0+x -> x
        // r2: (-x)+x -> 0
        // r3: (x+y)+z -> x+(y+z)
        let input_rule1 = "x: Int | plus![0 x] -> x";
        let input_rule2 = "x: Int | plus![minus!x x] -> 0";
        let input_rule3 =
            "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]";
        let rule1 = rule_parser(&types, &ctxts12, &opers)
            .parse(input_rule1)
            .unwrap()
            .0;
        let rule2 = rule_parser(&types, &ctxts12, &opers)
            .parse(input_rule2)
            .unwrap()
            .0;
        let rule3 = rule_parser(&types, &ctxts3, &opers)
            .parse(input_rule3)
            .unwrap()
            .0;
        let critical_pairs = make_critical_pair_set(&vec![rule1, rule2, rule3]);
        dbg!(&critical_pairs);
    }
}
