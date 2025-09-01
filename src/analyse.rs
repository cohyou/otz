use std::{cmp::Ordering, rc::Rc};

use crate::{context::Context, rule::Rule, term::TermInner};

pub fn analyse(context: Context, left: &Rc<TermInner>, right: &Rc<TermInner>) -> Rule {
    // analyse_inner(left.clone(), right.clone());

    // 1-1 順序つかないものを消す
    // let mut eqs = Vec::from(self.eqs.clone());
    // eqs.retain(|Equation(_,t1,t2,_)| {
    //     analyse2(t1.clone(), t2.clone()) || analyse2(t2.clone(), t1.clone())
    // });

    // 2-1 ルールそれぞれの後件を正規化する（新ルールを追加したルール群で）
    // let new_rule = if analyse2(new_pair.1.clone(), new_pair.2.clone()) {
    //     Rule::new(next_eq_id, new_pair.1, new_pair.2, (0, 0))
    // } else {
    //     Rule::new(next_eq_id, new_pair.2, new_pair.1, (0, 0))
    // };
    if analyse_inner(left, right) {
        // left > right
        Rule::new(context, left.clone(), right.clone())
    } else {
        // left < right
        Rule::new(context, right.clone(), left.clone())
    }
}

fn analyse_inner(t1: &Rc<TermInner>, t2: &Rc<TermInner>) -> bool {
    // まず、項のサイズを比較する
    // 一緒なら、関数のIDを比較する
    // 同じ関数なら、1つ目の引数同士を比較する
    match t1.size().cmp(&t2.size()) {
        Ordering::Greater => true, // left > right
        Ordering::Less => false, // left < right
        Ordering::Equal => {
            match (t1.as_ref(), t2.as_ref()) {
                (TermInner::Fun(f, args_f), TermInner::Fun(g, args_g)) => {
                    match f.cmp(g) {
                        Ordering::Greater => true, // f > g
                        Ordering::Less => false, // f < g
                        Ordering::Equal => {
                            analyse_inner(&args_f[0], &args_g[0])
                        }
                    }
                }
                _ => unimplemented!(),
            }            
        }
    }
}

impl TermInner {
    fn size(&self) -> usize {
        // 含まれる関数の数
        match &self {
            &TermInner::Var(_) | TermInner::RuledVar(_,_,_) => 0,
            &TermInner::Int(_) | TermInner::Str(_) => 1,
            &TermInner::Fun(_, args) => {
                1 + args.iter().map(|inner|inner.size()).sum::<usize>()
            },
            &TermInner::Subst(_, inner) => inner.size(),            
        }
    }
}

// fn lpo_gr(t1: Rc<TermInner>, t2: Rc<TermInner>) -> bool {
//     lpo_gr_eq(t1.clone(), t2.clone()) && !lpo_gr_eq(t2, t1)
// }

// fn lpo_gr_eq(t1: Rc<TermInner>, t2: Rc<TermInner>) -> bool {
//     match (t1.as_ref(), t2.as_ref()) {
//         (t, TermInner::Var(xi)) => occur(xi, t),
//         (TermInner::Var(_), _) => false,
//         (TermInner::Fun(f1, args1), TermInner::Fun(f2, args2)) => {
//             (
//                 f1 == f2 && 
//                 lex_gr_eq(lpo_gr_eq, args1.clone(), args2.clone()) && 
//                 args2.iter().all(|arg2| lpo_gr_eq(t1.clone(), arg2.clone()) ) 
//             ) ||
//             (
//                 f1 > f2 &&
//                 args2.iter().all(|arg2| lpo_gr(t1.clone(), arg2.clone()))
//             ) ||
//             args1.iter().any(|arg1| lpo_gr_eq(arg1.clone(), t2.clone()))
//         }
//         _ => unimplemented!()
//     }
// }
// fn occur(_v: &VarId, _t: &TermInner) -> bool { false }

// fn lex_gr_eq(gr_eq: fn(Rc<TermInner>, Rc<TermInner>) -> bool, ts1: Vec<Rc<TermInner>>, ts2: Vec<Rc<TermInner>>) -> bool {
//     match ts1.cmp(&ts2) {
//         Ordering::Greater => true,
//         Ordering::Less => false,
//         Ordering::Equal => {
//             for (x, y) in ts1.iter().zip(&ts2) {
//                 if gr_eq(x.clone(), y.clone()) && !gr_eq(y.clone(), x.clone()) {
//                     return true;
//                 } else if !(gr_eq(x.clone(), y.clone()) && gr_eq(y.clone(), x.clone())) {
//                     return false;
//                 }
//             }
//             true
//         }
//     }
// }

#[cfg(test)]
mod test {
    use combine::Parser;
    use rstest::*;

    use crate::{analyse::analyse, context_table::CtxtTable, id::{OperId, TypeId}, parser::equation::equation_parser, symbol_table::SymbolTable};

    #[rstest]
    #[case("x: Int | plus![0 x] = x")]
    #[case("x: Int | plus![minus!x x] = 0")]
    #[case("x: Int y: Int z: Int | plus![x plus![y z]] = plus![plus![x y] z]")]
    fn test_analyse(#[case] input: &str) {
        let types = SymbolTable::<TypeId>::new();
        types.assign("Int".to_string());
        let opers = SymbolTable::<OperId>::new();
        opers.assign("plus".to_string());
        opers.assign("minus".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("x".to_string());
        
        let eq = equation_parser(&types, &ctxts, &opers).parse(input).unwrap().0;
        dbg!(&eq);
        let rule = analyse(eq.context, &eq.left, &eq.right);
        dbg!(&rule);
    }
}