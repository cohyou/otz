use std::rc::Rc;

use crate::{equation::Equation, rule::Rule, subst::Subst, subterm::{Position, Subterm}, term::{Term, TermInner}, unify::unify};

pub fn complete(eqs: &Vec<Equation>) -> Vec<Rule> {
    let mut rules = eqs.iter().map(|eq| analyse(&eq.left, &eq.right)).collect::<Vec<_>>();
    let mut critical_pairs = make_critical_pair_set(&rules);

    while !critical_pairs.is_empty() {
        let (p, q) = critical_pairs.pop().unwrap();
        // p, qのrulesに関しての正規形p^,q^を求める
        let normal_p = p.normalize(&rules);
        let normal_q = q.normalize(&rules);
        if normal_p != normal_q {
            let new_rule = analyse(&normal_p.inner, &normal_q.inner);
            rules.push(new_rule.clone());
            // α→βと既存rules内のrule毎の危険対の集合を作る
            let new_pairs = rules.iter().flat_map(|rule| {
                find_critical_pairs(&new_rule, rule)
            }).collect::<Vec<_>>();
            critical_pairs.extend(new_pairs);
        }
    }
    rules
}

fn analyse(_left: &Rc<TermInner>, _right: &Rc<TermInner>) -> Rule {
    unimplemented!()
}

type CriticalPair = (Rc<Term>, Rc<Term>);

fn make_critical_pair_set(rules: &Vec<Rule>) -> Vec<CriticalPair> {
    // それぞれ2つのruleを取り出して、find_critical_pairsする
    // 同一ruleも対象
    rules.iter().flat_map(|rule1| {
        rules.iter().flat_map(|rule2| {
            rule1.check_overlap(rule2).iter()
                .map(Overlap::to_critical_pair).collect::<Vec<_>>()
        }).collect::<Vec<_>>()
    }).collect()
}

fn find_critical_pairs(rule1: &Rule, rule2: &Rule) -> Vec<CriticalPair> {
    // rule2とrule1との互いの危険対を探す
    // rule2とrule2・rule1とrule1の組み合わせは探さない
    let overlaps1 = rule1.check_overlap(rule2);
    let pairs1 = overlaps1.iter().map(Overlap::to_critical_pair);
    let overlaps2 = rule2.check_overlap(rule1);
    let pairs2 = overlaps2.iter().map(Overlap::to_critical_pair);
    pairs1.chain(pairs2).collect()
}

struct Overlap {
    pub overlapper: Rule,  // 重なる側
    pub overlappee: Rule,  // 重なられる側
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
    fn check_overlap(&self, from: &Rule) -> Vec<Overlap> {
        // fromがselfに重なるかだけを調べる、逆は行わない
        let s1 = self.before.clone();
        let s2 = from.before.clone();

        let is_same_rule = self == from;

        s1.subterms().filter_map(|subterm: Subterm| {
            let s1_sub = subterm.term;
            // s1/uが変数なら対象外
            let s1_is_not_var = !matches!(s1_sub.inner.as_ref(), TermInner::Var(_));
            // r1≡r2ならu≠ε(恒等写像=無意味な置換になってしまう)
            let is_not_identity = is_same_rule && !subterm.pos.is_empty();

            // 上記を前提とする
            (s1_is_not_var && is_not_identity).then(|| {
                unify(s1_sub.inner.clone(), s2.inner.clone())
                .map(|theta| {
                    (subterm.pos, theta)
                })
            }).flatten()
        }).map(|(pos, theta)| {
            Overlap {
                overlapper: from.clone(),
                overlappee: self.clone(),
                pos: pos,
                subst: theta
            }
        }).collect()
    }
}

impl Overlap {
    /// r1:s1->t1 r2:s2->t2
    /// r2がuでmguθによりr1に重なるとする
    /// その場合<θs1[u<-t2], θt1>をr1とr2の危険対という
    /// ちなみに、一般性を失わず、s1とs2の変数は重ならないとしてよい
    fn to_critical_pair(&self) -> CriticalPair {
        // 重なりを危険対に変換する

        // θs1[u<-t2]
        let s1 = self.overlappee.clone().before;        
        let to = self.overlapper.clone().after;
        let left = s1.substitute(&self.subst).replace(&self.pos, to);

        // θt1
        let t1 = self.overlappee.clone().after;
        let right = Rc::new(t1.substitute(&self.subst));
        (left, right)
    }
}