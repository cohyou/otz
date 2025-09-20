use std::{
    collections::{BinaryHeap, HashMap},
    rc::Rc,
};

use crate::{
    completion::complete,
    critical_pairs::prepare_rules,
    equation::Equation,
    instance::Instance,
    rule::Rule,
    subst::{Subst, Var},
    subterm::{Position, SubTerm},
    term::{Term, TermInner},
};

impl Instance {
    pub fn deducible(&self, eq: &Equation) -> bool {
        // TODO: 本来はSchemaのconstraintsも必要
        // let eqs = self.data.iter().map(Equation::to_rule).collect();
        let eqs = &self.data;
        let rules = complete(eqs, 0);
        eq.is_reducible(&rules)
    }
}

impl Equation {
    pub fn to_rule(&self) -> Rule {
        // TODO: 本来はそのままではなく向きを決めるアルゴリズムの実装が必要
        Rule::new(
            self.context.clone(),
            self.names.clone(),
            self.left.clone(),
            self.right.clone(),
        )
    }

    fn is_reducible(&self, rules: &BinaryHeap<Rule>) -> bool {
        self.left_term().normalize(rules) == self.right_term().normalize(rules)
    }
}

impl Term {
    pub fn normalize(&self, rules: &BinaryHeap<Rule>) -> Rc<Term> {
        let mut term = Rc::new(self.clone());
        loop {
            let result = reduct(term.clone(), rules);
            // println!("result: {} term: {}", result, term);
            if result == term {
                break;
            }
            // println!("REDUCED {} -> {}", term, result);
            term = result
        }
        // println!("normalized: {}", &term);
        term
    }
}

#[derive(Debug)]
pub struct Redex {
    pub term: Rc<Term>,
    pub pos: Position,
    pub subst: Subst,
    pub rule: Rule,
}

impl std::fmt::Display for Redex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Redex<\n.       term: {}, \n.       pos: {:?}, \n.       subst: {:?}, \n.       rule: {}>",
            self.term, self.pos, self.subst, self.rule
        )
    }
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

pub fn reduct(term: Rc<Term>, rules: &BinaryHeap<Rule>) -> Rc<Term> {
    let prepared_rules = prepare_rules(rules)
        .iter()
        .map(|rule| rule.make_vars_ruled(crate::rule::RuleKind::NotSet))
        .collect::<Vec<_>>(); // dbg!(&prepared_rules);
    let redexes = prepared_rules
        .iter()
        .map(|rule| term.find_redexes_from(rule))
        .flatten()
        .collect::<Vec<_>>();

    // crate::util::dispv(&redexes);

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
        // println!("find_redexes_from self: {} rule: {}", self, rule);
        self.subterms()
            // .inspect(|subterm| {
            //     dbg!(subterm);
            // })
            .filter_map(|subterm| {
                // `σs`が`subterm`に一致するような`σ`があるかどうかを探す
                subterm.find_redex_from(rule)
            })
            .collect()
    }
}

impl SubTerm {
    // left_termをselfに変換するための代入があればそれを返す
    // この結果が複数存在するかどうか理解できていないが、0 or 1だと仮定する
    fn find_redex_from(&self, rule: &Rule) -> Option<Redex> {
        // println!("find_redex_from self: {:?} rule: {}", self, rule);

        // 部分項(pos, term)に対して、σsがtermに一致するかどうか
        // そのような出現位置をuとするとt/u ≡ σsとなる。

        // σsがtermに一致するような代入σが存在するか？
        let pattern = rule.before();
        let maybe_subst = pattern.inner.try_match2(self.term.inner.clone());
        maybe_subst
            .map(|subst| Redex::new(self.main.clone(), self.pos.clone(), subst, rule.clone()))
    }
}

impl Var {
    fn is_used_in(&self, term: Rc<TermInner>) -> bool {
        match (self, term.as_ref()) {
            (Var::Id(vid), _) => term.vars().contains(&Var::Id(vid.clone())),
            (Var::Ruled(vid, rid, kind), _) => {
                term.vars()
                    .contains(&Var::Ruled(vid.clone(), *rid, kind.clone()))
            }
        }
    }
}

impl TermInner {
    /// selfはパターン
    fn try_match2(&self, term: Rc<TermInner>) -> Option<Subst> {
        // println!("try_match2 pat: {:?} term: {:?}", self, term);
        match self {
            TermInner::Var(vid) => {
                // println!("try_match2 var pat: {:?} term: {:?}", self, term);
                match term.as_ref() {
                    TermInner::Var(v) if vid == v => Some(Subst::default()),
                    _ => {
                        // ここは、以下の前提が存在している
                        // パターン側変数がRuledVar、項の変数がVarであること
                        // かつ、matchとして、項の変数はある種、定数扱いであり、項変数に何かを代入することは考えていないこと
                        // unifyとはそこが違う（＝代入の方向が一方的）
                        None

                        // (!Var::Id(vid.clone()).is_used_in(term.clone())).then(|| {
                        //     let new_subst = HashMap::from([(Var::Id(vid.clone()), term.clone())]);
                        //     // subst.insert(Var::Id(vid.clone()), term.clone());
                        //     new_subst.into()
                        // })
                    }
                }
            }
            TermInner::RuledVar(vid, rid, kind) => {
                // println!("try_match2 ruledvar pat: {} term: {}", self, term);
                match term.as_ref() {
                    TermInner::RuledVar(v, r, k) if vid == v && rid == r && kind == k => {
                        Some(Subst::default())
                    }
                    _ => {
                        (!Var::Ruled(vid.clone(), *rid, kind.clone()).is_used_in(term.clone()))
                            .then(|| {
                                let subst = HashMap::from([(
                                    Var::Ruled(vid.clone(), *rid, kind.clone()),
                                    term.clone(),
                                )]);
                                subst.into()
                                // subst.insert(Var::Ruled(vid.clone(), *rid, kind.clone()), term.clone());
                                // subst
                            })
                    }
                }
            }
            TermInner::Fun(oid_pat, args_pat) => {
                if let TermInner::Fun(oid_tgt, args_tgt) = term.as_ref() {
                    (oid_pat == oid_tgt && args_pat.len() == args_tgt.len())
                        .then(|| {
                            let subst = Subst::default();
                            args_pat
                                .iter()
                                .zip(args_tgt)
                                .try_fold(subst, |mut subst, (pt, tg)| {
                                    let new_pat = pt.substitute(&subst);
                                    new_pat.try_match2(tg.clone()).map(|new_subst| {
                                        // subst.0.extend(new_subst.0);
                                        subst = new_subst.compose(&subst);
                                        subst
                                    })
                                })
                        })
                        .flatten()
                } else {
                    None
                }
            }
            _ => (self == term.as_ref()).then_some(Subst::default()),
        }
    }
}

impl TermInner {
    pub fn vars(&self) -> Vec<Var> {
        match self {
            TermInner::Var(var) => vec![Var::Id(var.clone())],
            TermInner::RuledVar(vid, rid, kind) => {
                vec![Var::Ruled(vid.clone(), *rid, kind.clone())]
            }
            TermInner::Fun(_, args) => args.iter().map(|arg| arg.vars()).flatten().collect(),
            _ => vec![],
        }
    }
}

impl Term {
    /// selfはパターン
    #[allow(dead_code)]
    fn try_match(&self, term: Rc<Term>) -> Option<Subst> {
        println!("try_match pat: {} term: {}", self, term);
        match self.inner.as_ref() {
            TermInner::Var(vid) => {
                // println!("try_match var pat: {:?} term: {:?}", self, term);
                match term.inner.as_ref() {
                    TermInner::Var(v) if vid == v => Some(Subst::default()),
                    _ => (!Var::Id(vid.clone()).is_used_in(term.inner.clone())).then(|| {
                        let subst = HashMap::from([(Var::Id(vid.clone()), term.inner.clone())]);
                        subst.into()
                    }),
                }
            }
            TermInner::RuledVar(vid, rid, kind) => {
                // println!("try_match ruledvar pat: {} term: {}", self, term);
                match term.inner.as_ref() {
                    TermInner::RuledVar(v, r, k) if vid == v && rid == r && kind == k => {
                        Some(Subst::default())
                    }
                    _ => (!Var::Ruled(vid.clone(), *rid, kind.clone())
                        .is_used_in(term.inner.clone()))
                    .then(|| {
                        let subst = HashMap::from([(
                            Var::Ruled(vid.clone(), *rid, kind.clone()),
                            term.inner.clone(),
                        )]);
                        subst.into()
                    }),
                }
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
mod tests {
    use std::rc::Rc;

    use combine::EasyParser;
    use rstest::*;

    use crate::context_table::CtxtTable;
    use crate::parser::term::term_parser;
    use crate::reduct::reduct;
    use crate::rule::Rule;
    use crate::util::{opers, rules, tm, types};

    #[rstest]
    #[case("x z: Int | plus![minus!x plus![x z]]", rule123())]
    #[case("x y z: Int | plus![minus!x plus![plus![x y] z]]", rule34())]
    #[case("x z: Int | plus![0 plus![x z]]", rule123())]
    #[case(
        "x y z: Int | plus![minus!plus![x y] plus![x plus![y z]]]",
        rule123456()
    )]
    #[case(
        "x y z: Int | plus![minus!x plus![minus!y plus![plus![y x] z]]]",
        rule1to12()
    )]
    fn test_normalize(#[case] input: &str, #[case] rules: Vec<Rule>) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let term = tm(input, &types, &opers, &ctxts);
        let normalized = term.normalize(&rules.into());
        println!("normalized: {}", normalized);
    }

    #[rstest]
    #[case("x z: Int | plus![0 plus![x z]]", rule123())]
    #[case("xx yy zz z: Int | plus![plus![xx plus![yy zz]] z]", rule123())]
    #[case("x z: Int | plus![minus!x plus![x z]]", rule123())]
    #[case("x y z: Int | plus![minus!x plus![x plus![y z]]]", rule34())]
    #[case("x y: Int | plus![minus!plus![0 x] plus![x y]]", rule1to8())]
    #[case(
        "x y z: Int | plus![minus!x plus![minus!y plus![y plus![x z]]]]",
        rule1to12()
    )]
    fn test_reduct(#[case] input: &str, #[case] rules: Vec<Rule>) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let term = tm(input, &types, &opers, &ctxts);
        let reducted = reduct(Rc::new(term), &rules.into());
        println!("reducted: {}", reducted);
    }

    #[rstest]
    #[case("x z: Int | plus![0 plus![x z]]")]
    fn test_find_redexes_from(#[case] input: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let term = term_parser(&types, &opers, &ctxts)
            .easy_parse(input)
            .unwrap()
            .0;
        let redexes = term.find_redexes_from(&rule123()[0]);
        redexes.iter().for_each(|redex| {
            println!("{}", redex);
        });
    }

    #[rstest]
    #[case(
        "x z: Int | plus![0 plus![x z]]", vec![],
        "x: Int | plus![0 x] -> x")]
    #[case(
        "xx yy zz z: Int | plus![plus![xx yy] plus![zz z]]", vec![],
        "x y z: Int | plus![plus![x y] z] -> plus![x plus![y z]]")]
    #[case(
        "x y z: Int | plus![minus!x plus![plus![x y] z]]", vec![1], 
        "x y z: Int | plus![plus![x y] z] -> plus![x plus![y z]]")]
    #[case(
        "x y z: Int | plus![minus!x plus![x plus![y z]]]", vec![], 
        "x y: Int | plus![minus!x plus![x y]] -> y")]
    #[case(
        "x y z: Int | plus![minus!plus![x y] plus![x plus![y z]]]", vec![], 
        "x: Int | plus![minus!x x] -> 0")]
    fn test_find_redex_from(
        #[case] term_input: &str,
        #[case] pos: Vec<usize>,
        #[case] rule_input: &str,
    ) {
        use crate::{
            critical_pairs::prepare_rules,
            subterm::SubTerm,
            util::{rl, tm},
        };

        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();

        let term = Rc::new(tm(term_input, &types, &opers, &ctxts));
        let subterm = SubTerm {
            main: term.clone(),
            pos: pos.clone(),
            term: term.get_at(&pos).unwrap(),
        };
        let rule = rl(rule_input, &types, &opers, &ctxts);
        let rule =
            &prepare_rules(&vec![rule].into())[0].make_vars_ruled(crate::rule::RuleKind::NotSet);
        let redexes = subterm.find_redex_from(&rule);
        println!("redexes: {}", &redexes.unwrap());
    }

    #[rstest]
    #[case("x: Int | plus![0 x]]", "x: Int z: Int | plus![0 plus![x z]]")]
    #[case("x: Int | x", "x: Int z: Int | plus![x z]")]
    #[case("x: Int | 0", "x: Int | 0")]
    fn test_try_match(#[case] pattern: &str, #[case] term: &str) {
        use crate::parser::term::term_parser;
        use combine::Parser;

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

    fn rule1to12() -> Vec<Rule> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        rules(
            &types,
            &opers,
            &ctxts,
            vec![
            "x: Int | plus![0 x] -> x",
            "x: Int | plus![minus!x x] -> 0",
            "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
            "x y: Int | plus![minus!x plus![x y]] -> y",
            "x: Int | plus![minus!0 x] -> x",
            "x: Int | plus![minus!minus!x 0] -> x",
            "x y z: Int | plus![minus!plus![x y] plus![x plus![y z]]] -> z",
            "x y: Int | plus![minus!minus!x y] -> plus![x y]",
            "x y: Int | plus![minus!plus![minus!plus![x y] x] 0] -> y",
            "x y: Int | plus![minus!plus![x minus!y] plus![x 0]] -> y",
            "x y z v3: Int | plus![minus!plus![x plus![y z]] plus![x plus![y plus![z v3]]]] -> v3",
        ],
        )
    }

    fn rule1to8() -> Vec<Rule> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        rules(
            &types,
            &opers,
            &ctxts,
            vec![
                "x: Int | plus![0 x] -> x",
                // "x: Int | plus![minus!x x] -> 0",
                // "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
                // "x y: Int | plus![minus!x plus![x y]] -> y",
                // "x: Int | plus![minus!0 x] -> x",
                // "x: Int | plus![minus!minus!x 0] -> x",
                // "x y z: Int | plus![minus!plus![x y] plus![x plus![y z]]] -> z",
                // "x y: Int | plus![minus!minus!x y] -> plus![x y]",
            ],
        )
    }

    fn rule123456() -> Vec<Rule> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        rules(
            &types,
            &opers,
            &ctxts,
            vec![
                "x: Int | plus![0 x] -> x",
                "x: Int | plus![minus!x x] -> 0",
                "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
                "x y: Int | plus![minus!x plus![x y]] -> y",
                "x: Int | plus![minus!0 x] -> x",
                "x: Int | plus![minus!minus!x 0] -> x",
            ],
        )
    }

    /// r1: 0+x -> x
    /// r2: (-x)+x -> 0
    /// r3: (x+y)+z -> x+(y+z)
    fn rule123() -> Vec<Rule> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        rules(
            &types,
            &opers,
            &ctxts,
            vec![
                "x: Int | plus![0 x] -> x",
                "x: Int | plus![minus!x x] -> 0",
                "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
            ],
        )
    }

    /// r3: (x+y)+z -> x+(y+z)
    /// r4: (-x)+(x+y) -> y
    fn rule34() -> Vec<Rule> {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["plus", "minus"]);
        let ctxts = CtxtTable::new();
        rules(
            &types,
            &opers,
            &ctxts,
            vec![
                "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
                "x y: Int | plus![minus!x plus![x y]] -> y",
            ],
        )
    }
    
    // fn rule3() -> Vec<Rule> {
    //     let types = types(vec!["Int"]);
    //     let opers = opers(vec!["plus", "minus"]);
    //     let ctxts = CtxtTable::new();
    //     rules(&types, &ctxts, &opers, vec![
    //         "x: Int y: Int z: Int | plus![plus![x y] z] -> plus![x plus![y z]]",
    //     ])
    // }
}
