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
            stack: vec![Subterm { main: Rc::new(self.clone()), pos: vec![], term: Rc::new(self.clone()) }],
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
        None
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

        if let TermInner::Fun(_, args) = current.term.inner.as_ref() {
            for (i, _child) in args.iter().enumerate().rev() {
                let mut next_pos = current.pos.clone();
                next_pos.push(i);
                // self.stack.push(SubtermRef { pos: next_pos, term: child });
            }
        }

        Some(current)
    }
}

/// ─────────────────────────────────────────────────────────
/// 使い方イメージ
/// ─────────────────────────────────────────────────────────
#[cfg(test)]
mod demo {
//     use super::*;

//     #[test]
//     fn enumerate_subterms() {
//         // f(g(x), h(a, b))
//         let t = Term::fun(
//             "f",
//             vec![
//                 Term::fun("g", vec![Term::var("x")]),
//                 Term::fun("h", vec![Term::var("a"), Term::var("b")]),
//             ],
//         );

//         // 前順（トップダウン）
//         let got: Vec<(Position, String)> = t
//             .subterms()
//             .map(|st| (st.pos, format!("{:?}", st.term)))
//             .collect();

//         // 例）最初は全体（位置[]）、次に [0] = g(x)、[0,0] = x、[1] = h(a,b)、[1,0] = a、[1,1] = b
//         assert_eq!(got[0].0, vec![]);
//         assert_eq!(got[1].0, vec![0]);
//         assert_eq!(got[2].0, vec![0, 0]);
//         assert_eq!(got[3].0, vec![1]);
//         assert_eq!(got[4].0, vec![1, 0]);
//         assert_eq!(got[5].0, vec![1, 1]);

//         // 位置アクセス
//         let sub = t.get_at(&[1, 0]).unwrap(); // a
//         assert!(matches!(sub, Term::Var(s) if s == "a"));
//     }
}
