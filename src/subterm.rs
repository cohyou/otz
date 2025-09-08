use std::rc::Rc;

use crate::{subst::Var, term::{Term, TermInner}};

/// ─────────────────────────────────────────────────────────
/// ざっくりTRS用：項の定義と部分項イテレータ
/// ─────────────────────────────────────────────────────────

pub type Position = Vec<usize>;

#[derive(Clone, Debug)]
pub struct Subterm {
    pub main: Rc<Term>,
    pub pos: Position,
    pub term: Rc<Term>,
}

impl Term {
    /// 前順（トップダウン、左→右）で部分項を走査するイテレータ。
    pub fn subterms(&self) -> Subterms {
        Subterms {
            stack: vec![Subterm {
                main: Rc::new(self.clone()),
                pos: vec![],
                term: Rc::new(self.clone()),
            }],
        }
    }

    /// 位置で部分項を取得（見つからなければ None）
    pub fn get_at(&self, pos: &[usize]) -> Option<Rc<Term>> {
        let mut t = &self.inner;
        for &i in pos {
            match t.as_ref() {
                TermInner::Fun(_, args) => t = args.get(i)?,
                TermInner::Var(_) => return None,
                _ => return None,
            }
        }
        Some(Rc::new(Term {
            context: self.context.clone(),
            names: self.names.clone(),
            inner: t.clone(),
        }))
    }

    pub fn vars(&self) -> Vec<Var> {
        self.subterms().filter_map(|subterm| {
            match subterm.term.inner.as_ref() {
                TermInner::Var(var) => Some(Var::Id(var.clone())),
                TermInner::RuledVar(vid, rid, kind) => {
                    Some(Var::Ruled(vid.clone(), *rid, kind.clone()))
                },
                _ => None
            }
        }).collect()
    }

    // /// 後順（ボトムアップ）をざっくり：前順を全部集めて反転
    // pub fn subterms_postorder(&self) -> impl Iterator<Item = Subterm> {
    //     let mut v: Vec<_> = self.subterms().collect();
    //     v.reverse();
    //     v.into_iter()
    // }
}

/// 内部はシンプルにLIFOスタック（DFS前順）。子は逆順でpushして左→右を維持。
pub struct Subterms {
    stack: Vec<Subterm>,
}

impl<'a> Iterator for Subterms {
    type Item = Subterm;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;

        let current_term = current.term.clone();
        if let TermInner::Fun(_, args) = current_term.inner.as_ref() {
            for (i, child) in args.iter().enumerate().rev() {
                let mut next_pos = current.pos.clone();
                next_pos.push(i);
                let t = current.clone().term;
                let child_term = Term {
                    context: t.clone().context.clone(),
                    names: t.clone().names.clone(),
                    inner: child.clone(),
                };
                self.stack.push(Subterm {
                    main: current.term.clone(),
                    pos: next_pos,
                    term: Rc::new(child_term),
                });
            }
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::term::term_parser;
    use crate::util::{opers, types};
    use crate::{context_table::CtxtTable, subterm::Position};

    #[test]
    fn test_subterms1() {
        use crate::combine::EasyParser;

        let input = "a: Int | f![f!a a f;]";

        let types = types(vec!["Int"]);
        let opers = opers(vec!["f"]);
        let ctxts = CtxtTable::new();
        let term = term_parser(&types, &opers, &ctxts)
            .easy_parse(input)
            .unwrap()
            .0;

        let got: Vec<(Position, String)> = term
            .subterms()
            .map(|st| (st.pos, format!("{:?}", st.term)))
            .collect();
        dbg!(&got);
    }
}
