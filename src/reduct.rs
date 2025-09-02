use std::{collections::HashMap, rc::Rc};

use crate::{
    completion::complete,
    equation::Equation,
    instance::Instance,
    rule::Rule,
    subst::{Subst, Var},
    subterm::{Position, Subterm},
    term::{Term, TermInner},
};

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
        Rule::new(self.context.clone(), self.left.clone(), self.right.clone())
    }

    fn is_reducible(&self, rules: &Vec<Rule>) -> bool {
        self.left_term().normalize(rules) == self.right_term().normalize(rules)
    }
}

impl Term {
    pub fn normalize(&self, rules: &Vec<Rule>) -> Rc<Term> {
        let mut term = Rc::new(self.clone());
        loop {
            let result = reduct(term.clone(), rules);
            if result == term {
                break;
            }
            term = result
        }
        term
    }
}

#[derive(Debug)]
pub struct Redex {
    term: Rc<Term>,
    pos: Position,
    subst: Subst,
    rule: Rule,
}

impl Redex {
    fn new(term: Rc<Term>, pos: Position, subst: Subst, rule: Rule) -> Self {
        Redex {
            term,
            pos,
            subst,
            rule,
        }
    }
}

pub fn reduct(term: Rc<Term>, rules: &Vec<Rule>) -> Rc<Term> {
    let redexes = rules
        .iter()
        .map(|rule| term.find_redexes_from(rule))
        .flatten()
        .collect::<Vec<_>>();

    dbg!(&redexes);
    if redexes.is_empty() {
        return term.clone();
    }

    // TODO: ひとまず戦略は後で考える
    let redex = &redexes[0];

    // 置き換え後の項を作成する
    redex.apply()
}

impl Term {
    /// まず、とある項`self`があり、規則`rule:before->after`があるとする。
    /// `self`の部分項のうち、`σ(before)`と一致するような`σ`が存在するようなものを探す。
    fn find_redexes_from(&self, rule: &Rule) -> Vec<Redex> {
        self.subterms()
            .inspect(|subterm| {
                dbg!(subterm);
            })
            .filter_map(|subterm| {
                // `σs`が`subterm`に一致するような`σ`があるかどうかを探す
                subterm.find_redex_from(rule)
            })
            .collect()
    }
}

impl Redex {
    /// 項termの部分項self/atをtoで置き換えた項`self[ at <- to ]`を得る。
    pub fn apply(&self) -> Rc<Term> {
        let to = self.rule.after().substitute(&self.subst);
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
        let pattern = rule.before();
        let init = Subst::default();

        pattern
            .subterms()
            .try_fold(init, |mut subst, pat_subterm| {
                dbg!(&pat_subterm.pos);
                self.term.get_at(&pat_subterm.pos).and_then(|t| {
                    // これまでのsubstを適用する
                    let new_t = t.substitute(&subst);
                    let pat = pat_subterm.term;
                    dbg!(&pat, &t, &new_t);
                    pat.try_match(Rc::new(new_t)).map(|new_subst| {
                        subst.0.extend(new_subst.0);
                        dbg!(&subst);
                        subst
                    })
                })
            })
            .map(|subst| Redex::new(self.main.clone(), self.pos.clone(), subst, rule.clone()))
    }
}

impl Term {
    /// selfはパターン
    fn try_match(&self, term: Rc<Term>) -> Option<Subst> {
        match self.inner.as_ref() {
            TermInner::Var(vid) => {
                let subst = HashMap::from([(Var::Id(vid.clone()), term.inner.clone())]);
                Some(subst.into())
            }
            TermInner::RuledVar(vid, rid, kind) => {
                let subst = HashMap::from([(
                    Var::Ruled(vid.clone(), *rid, kind.clone()),
                    term.inner.clone(),
                )]);
                Some(subst.into())
            }
            TermInner::Fun(oid_pat, _) => {
                if let TermInner::Fun(oid_tgt, _) = term.inner.as_ref() {
                    (oid_pat == oid_tgt).then_some(Subst::default())
                } else {
                    None
                }
            }
            _ => (self.inner == term.inner).then_some(Subst::default()),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{rc::Rc};

    use combine::EasyParser;
    use rstest::*;

    use crate::parser::rule::rule_parser;
    use crate::parser::term::term_parser;
    use crate::reduct::reduct;
    use crate::rule::Rule;
    use crate::util::{opers, types};
    use crate::{
        context_table::CtxtTable,
    };

    #[rstest]
    #[case("xx yy zz z: Int | plus![plus![xx plus![yy zz]] z]")]
    fn test_normalize(#[case] input: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus"]);    
        let ctxts = CtxtTable::new();

        let term = term_parser(&types, &opers, &ctxts)
            .easy_parse(input)
            .unwrap()
            .0;
        let normalized = term.normalize(&rules());
        dbg!(normalized);
    }

    #[rstest]
    #[case("x: Int | plus![0 x]]", "x: Int z: Int | plus![0 plus![x z]]")]
    #[case("x: Int | x", "x: Int z: Int | plus![x z]")]
    #[case("x: Int | 0", "x: Int | 0")]
    fn test_try_match(#[case] pattern: &str, #[case] term: &str) {
        use combine::Parser;
        use crate::parser::term::term_parser;

        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus"]);        
        let ctxts = CtxtTable::new();

        let pattern = term_parser(&types, &opers, &ctxts)
            .parse(pattern)
            .unwrap()
            .0;
        let term = term_parser(&types, &opers, &ctxts).parse(term).unwrap().0;

        let result = pattern.try_match(Rc::new(term));
        dbg!(&result);
    }

    #[rstest]
    #[case("x: Int | plus![0 x] -> x", vec![], "x z: Int | plus![0 plus![x z]]")]
    #[case("x y z: Int | plus![plus![x y] z] -> plus![x plus![y z]]", vec![], "xx yy zz z: Int | plus![plus![xx yy] plus![zz z]]")]
    fn test_find_redex_from(#[case] rule: &str, #[case] pos: Vec<usize>, #[case] input: &str) {
        use crate::subterm::Subterm;

        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus"]);        
        let ctxts = CtxtTable::new();

        let term = Rc::new(
            term_parser(&types, &opers, &ctxts)
                .easy_parse(input)
                .unwrap()
                .0,
        );
        let subterm = Subterm {
            main: term.clone(),
            pos: pos.clone(),
            term: term.get_at(&pos).unwrap(),
        };
        let rule = rule_parser(&types, &ctxts, &opers)
            .easy_parse(rule)
            .unwrap()
            .0;
        let redexes = subterm.find_redex_from(&rule);
        dbg!(&redexes);
    }

    #[rstest]
    #[case("x z: Int | plus![0 plus![x z]]")]
    fn test_find_redexes_from(#[case] input: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus"]);
        let ctxts = CtxtTable::new();

        let term = term_parser(&types, &opers, &ctxts).easy_parse(input).unwrap().0;
        let redexes = term.find_redexes_from(&rules()[0]);
        dbg!(&redexes);
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
            .easy_parse(input_rule1)
            .unwrap()
            .0;
        let rule2 = rule_parser(&types, &ctxts, &opers)
            .easy_parse(input_rule2)
            .unwrap()
            .0;
        let rule3 = rule_parser(&types, &ctxts, &opers)
            .easy_parse(input_rule3)
            .unwrap()
            .0;
        vec![rule1, rule2, rule3]
    }

    #[rstest]
    #[case("x z: Int | plus![0 plus![x z]]")]
    #[case("xx yy zz z: Int | plus![plus![xx plus![yy zz]] z]")]
    fn test_reduct(#[case] input: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus"]);
        let ctxts = CtxtTable::new();

        let term = term_parser(&types, &opers, &ctxts)
            .easy_parse(input)
            .unwrap()
            .0;
        let reducted = reduct(Rc::new(term), &rules());
        dbg!(reducted);
    }
}
