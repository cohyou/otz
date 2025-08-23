use std::{collections::HashMap, rc::Rc};

use crate::{id::VarId, subst::Subst, term::{TermInner}};

/// TODO: s/tのcontextの扱いを確認する
pub fn unify(s: Rc<TermInner>, t: Rc<TermInner>) -> Option<Subst> {
    use TermInner::{Var, Int, Str, Fun};
    
    match (s.as_ref(), t.as_ref()) {
        // 全く同じ内容なら
        (Var(x), Var(y)) if x == y => None,
        (Int(x), Int(y)) if x == y => None,
        (Str(s1), Str(s2)) if s1 == s2 => None,

        // s,tのどちらかが変数
        // 変数をx, 他の項をuとする
        (Var(x), u) |
        (u, Var(x)) => 
            is_subterm_of(x, u).then_some(
                HashMap::from([(x.clone(), Rc::new(u.clone()))]).into()
            ),

        // s,tが関数
        (Fun(oid_f, args_f), Fun(oid_g, args_g))  => {
            // t≡f(t1,...,tn),s≡g(s1,...,sm)とする
            // 前提: fとgが同じ関数 and 引数の数も同じ f ≡ g || m = n
            let is_same_oper = oid_f == oid_g && args_f.len() == args_g.len();

            is_same_oper.then(|| {             
                let init = Subst::default();
                args_f.iter().zip(args_g).try_fold(init,|theta, (tk, sk)| {
                    // 代入適用は演算子で定義してもいいかも theta[tk], theta[sk]
                    let s = sk.substitute(&theta);
                    let t = tk.substitute(&theta);
                    unify(t, s).map(|mut sigma| {
                        // 合成は演算子で定義してもいいかも theta = sigma[theta];
                        sigma.compose(theta)
                    })
                })
            }).flatten()
        }             
        _ => unimplemented!(),
    }
}

impl Subst {
    // 代入の合成を行う
    fn compose(&mut self, other: Subst) -> Subst {
        self.0.extend(other.0);
        Subst(self.0.clone())
    }
}

// impl std::ops::Index<Subst> for Subst {
//     type Output = Subst;
//     fn index<'a>(&'a self, index: Subst) -> &'a Self::Output {
//         &self.compose(index)
//     }
// }

/// xがuの真部分項かどうか
/// subtermsを調べていけばわかりそう
/// 早いかどうかは別
fn is_subterm_of(_vid: &VarId, _target: &TermInner) -> bool {
    // target.subterms().any(|t| t.term.as_ref() == self)
    false
}