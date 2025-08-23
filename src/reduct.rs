use std::{collections::HashMap, rc::Rc};

use crate::{completion::complete, equation::Equation, instance::Instance, rule::Rule, subst::Subst, subterm::{Position, Subterm}, term::{Term, TermInner}};

impl Instance {
    pub fn deducible(&self, eq: &Equation) -> bool {
        // TODO: 本来はSchemaのconstraintsも必要
        // let eqs = self.data.iter().map(Equation::to_rule).collect();
        let eqs = &self.data;
        let rules = complete(eqs);
        eq.is_reducible(&rules)
    }
}

impl Equation {
    pub fn to_rule(&self) -> Rule {
        // TODO: 本来はそのままではなく向きを決めるアルゴリズムの実装が必要
        Rule { before: Rc::new(self.left_term()), after: Rc::new(self.right_term()) }
    }

    fn is_reducible(&self, rules: &Vec<Rule>) -> bool {
        self.left_term().normalize(rules) == self.right_term().normalize(rules)
    }
}

impl Term {
    pub fn normalize(&self, rules: &Vec<Rule>) -> Rc<Term> {
        let mut term = Rc::new(self.clone());
        loop {
            let result = term.reduct(rules);
            if result == term { break; }
            term = result
        }
        term
    }
}

pub struct Redex {
    term: Rc<Term>,
    pos: Position,
    subst: Subst,
    rule: Rule,
}

impl Redex {
    fn new(term: Rc<Term>, pos: Position, subst: Subst, rule: Rule) -> Self {
        Redex { term, pos, subst, rule }
    }
}

impl Term {
    pub fn reduct(&self, rules: &Vec<Rule>) -> Rc<Term> {
        let redexes = rules.iter().map(|rule| {
            self.find_redexes_from(rule)
        }).collect::<Vec<_>>();

        // TODO: ひとまず戦略は後で考える
        let redex = &redexes[0][0];

        // 置き換え後の項を作成する
        redex.apply()
    }

    /// まず、とある項`self`があり、規則`rule:before->after`があるとする。
    /// `self`の部分項のうち、`σ(before)`と一致するような`σ`が存在するようなものを探す。
    fn find_redexes_from(&self, rule: &Rule) -> Vec<Redex> {
        self.subterms().filter_map(|subterm| {
            // `σs`が`subterm`に一致するような`σ`があるかどうかを探す
            subterm.find_redex_from(rule)
        })
        .collect()
    }
}

impl Redex {
    /// 項termの部分項self/atをtoで置き換えた項`self[ at <- to ]`を得る。
    pub fn apply(&self) -> Rc<Term> {
        let to = self.rule.after.substitute(&self.subst);
        self.term.replace(&self.pos, to.into())
    }
}

impl Subterm {
    // left_termをselfに変換するための代入があればそれを返す
    // この結果が複数存在するかどうか理解できていないが、0 or 1だと仮定する
    fn find_redex_from(&self, rule: &Rule) -> Option<Redex> {
        
        // 部分項(pos, term)に対して、σsがtermに一致するかどうか
        // そのような出現位置をuとするとt/u ≡ σsとなる。

        // σsがtermに一致するような代入σが存在するか？
        let pattern = rule.clone().before;        
        let mut matching_iterator = pattern.subterms().zip(self.term.subterms());
        let init = Subst::default();
        matching_iterator.try_fold(init, |mut subst, (pat_subterm, subterm)| {
            subterm.term.get_at(&pat_subterm.pos)
            .and_then(|t| {
                // これまでのsubstを適用する
                let new_t = t.substitute(&subst);
                pat_subterm.term.try_match(Rc::new(new_t))
                .map(|new_subst| {
                    subst.0.extend(new_subst.0);
                    subst
                })
            })
        }).map(|subst| {
            Redex::new(self.main.clone(), self.pos.clone(), subst, rule.clone())
        })
    }
}

impl Term {
    fn try_match(&self, term: Rc<Term>) -> Option<Subst> {
        match self.inner.as_ref() {
            TermInner::Var(vid) => {
                let subst = HashMap::from([(vid.clone(), term.inner.clone())]);
                Some(subst.into())
            },
            TermInner::Fun(oid_pat, _) => {
                if let TermInner::Fun(oid_tgt, _) = term.inner.as_ref() {
                    (oid_pat == oid_tgt).then_some(Subst::default())
                } else {
                    None
                }
            }
            _ => (self == term.as_ref()).then_some(Subst::default())
        }
    }
}
