use std::rc::Rc;

use crate::term::{Term, TermInner};

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
    pub fn get_at(&self, _pos: &[usize]) -> Option<Rc<Term>> {
        //     let mut t = self;
        //     for &i in pos {
        //         match t {
        //             Term::Fun { args, .. } => t = args.get(i)?,
        //             Term::Var(_) => return None,
        //         }
        //     }
        //     Some(t)
        unimplemented!()
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
mod test {
    use crate::r#type::Type;
    use crate::{
        context::Context,
        context_table::CtxtTable,
        id::{OperId, TypeId, VarId},
        parser::term::terminner::terminner_parser_,
        subterm::Position,
        symbol_table::SymbolTable,
        term::Term,
    };
    use std::{collections::HashMap, rc::Rc};

    #[test]
    fn test_subterms1() {
        use crate::combine::EasyParser;

        let opers = SymbolTable::<OperId>::new();
        opers.assign("f".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("a".to_string());

        // let input = "f![f![a]]";
        let input = "f![f!a a f;]";
        let mut parser = terminner_parser_(&ctxts, &opers);

        let result = parser.easy_parse(input);
        dbg!(&result);
        assert!(result.is_ok());

        let terminner = result.unwrap().0;
        let context = Context(HashMap::from([(VarId(0), Type::Unary(TypeId(0)))]));
        let term = Term {
            context: context,
            inner: Rc::new(terminner),
        };
        let got: Vec<(Position, String)> = term
            .subterms()
            .map(|st| (st.pos, format!("{:?}", st.term)))
            .collect();
        dbg!(&got);
    }
}
