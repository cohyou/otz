// use core::panic;
use std::{cmp::Ordering, rc::Rc};

use crate::{context::Context, completion::rule::Rule, completion::subst::Var, symbol_table::Names, term::{TermInner}};

// enum PartialOrdering {
//     Greater,
//     Less,
//     Equal,
//     Incomparable,
// }

pub fn analyse(
    context: Rc<Context>,
    names: Rc<Names>,
    left: Rc<TermInner>,
    right: Rc<TermInner>,
) -> Option<Rule> {
    // println!("analyse left: {:?} right: {:?}", left, right);

    // analyse_inner(left.clone(), right.clone()).map(|cmp| {
    //     if cmp {
    //         // left > right
    //         Rule::new(context, names, left.clone(), right.clone())
    //     } else {
    //         // left < right
    //         Rule::new(context, names, right.clone(), left.clone())
    //     }
    // })

    if lpo_gr(left.clone(), right.clone()) {
        // left > right
        Some(Rule::new(context, names, left.clone(), right.clone()))
    } else {
        // left < right
        Some(Rule::new(context, names, right.clone(), left.clone()))
    }
}

// fn analyse_rpo(_t1: Rc<TermInner>, _t2: Rc<TermInner>) -> PartialOrdering {
//     PartialOrdering::Incomparable
    // match (t1.as_ref(), t2.as_ref()) {
    //     (TermInner::Fun(f1, args1), TermInner::Fun(f2, args2)) => {
    //         match f1.cmp(f2) {
    //             Ordering::Greater => Some(true), // f1 > f2
    //             Ordering::Less => Some(false),   // f1 < f2
    //             Ordering::Equal => {
    //                 if args1.len() != args2.len() {
    //                     panic!("args1.len() != args2.len()");
    //                 }
    //                 for (arg1, arg2) in args1.iter().zip(args2) {
    //                     match analyse_rpo(arg1, arg2) {
    //                         Some(true) => return Some(true),
    //                         Some(false) => return Some(false),
    //                         None => continue,
    //                     }
    //                 }
    //                 None
    //             }
    //         }
    //     }
    //     _ => unimplemented!(),
    // }
// }

#[allow(dead_code)]
fn analyse_inner(t1: Rc<TermInner>, t2: Rc<TermInner>) -> Option<bool> {
    // まず、項のサイズを比較する
    // 一緒なら、関数のIDを比較する
    // 同じ関数なら、1つ目の引数同士を比較する
    match t1.size().cmp(&t2.size()) {
        Ordering::Greater => Some(true), // left > right
        Ordering::Less => Some(false),   // left < right
        Ordering::Equal => {
            match (t1.as_ref(), t2.as_ref()) {
                (TermInner::Fun(f, args_f), TermInner::Fun(g, args_g)) => {
                    match f.cmp(g) {
                        Ordering::Greater => Some(true), // f > g
                        Ordering::Less => Some(false),   // f < g
                        Ordering::Equal => analyse_inner(args_f[0].clone(), args_g[0].clone()),
                    }
                }
                (TermInner::Int(_), TermInner::Fun(_, _)) => Some(false),
                // (TermInner::Var(_), TermInner::RuledVar(_, _, _)) => false,
                // (TermInner::RuledVar(_,_,_), TermInner::Var(_)) => true,
                _ => {
                    println!("t1: {:?} t2: {:?}", t1, t2);
                    None
                }
            }
        }
    }
}

impl TermInner {
    pub fn size(&self) -> usize {
        // 含まれる関数の数
        match &self {
            &TermInner::Var(_) | TermInner::RuledVar(_, _, _) => 0,
            &TermInner::Int(_) | TermInner::Str(_) => 1,
            &TermInner::Fun(_, args) => 1 + args.iter().map(|inner| inner.size()).sum::<usize>(),
            &TermInner::Subst(_) => unimplemented!(),
        }
    }
    pub fn var_size(&self) -> usize {
        // 含まれる変数の数
        match &self {
            &TermInner::Var(_) | TermInner::RuledVar(_, _, _) => 1,
            &TermInner::Int(_) | TermInner::Str(_) => 0,
            &TermInner::Fun(_, args) => {
                1 + args.iter().map(|inner| inner.var_size()).sum::<usize>()
            }
            &TermInner::Subst(_) => unimplemented!(),
        }
    }
}

/// 辞書式経路順序 (LPO) による厳密な比較 t1 >_lpo t2。
/// 演算子の優先順位は OperId の大小。Int/Str は全ての関数適用より小さい定数とみなし、
/// 異なる定数同士は比較不能とする。
fn lpo_gr(t1: Rc<TermInner>, t2: Rc<TermInner>) -> bool {
    use TermInner::{Fun, Int, RuledVar, Str, Var};
    match (t1.as_ref(), t2.as_ref()) {
        // 変数は何よりも大きくならない
        (Var(_) | RuledVar(_, _, _), _) => false,
        // s > x  ⇔  x が s に出現し、かつ s ≠ x（左が変数のケースは上で除外済み）
        (s, Var(x)) => occur(&crate::completion::subst::Var::Id(x.clone()), s),
        (s, RuledVar(x, rid, kind)) => {
            occur(&crate::completion::subst::Var::Ruled(x.clone(), *rid, kind.clone()), s)
        }
        (Fun(f, args1), t) => {
            // (a) ある引数 si が si ≥ t
            args1.iter().any(|si| lpo_gr_eq(si.clone(), t2.clone()))
                || match t {
                    Fun(g, args2) => {
                        let dominates_args = || args2.iter().all(|tj| lpo_gr(t1.clone(), tj.clone()));
                        if f > g {
                            // (b) f > g かつ全ての tj について s > tj
                            dominates_args()
                        } else if f == g && args1.len() == args2.len() {
                            // (c) 引数列が辞書式に大きく、かつ全ての tj について s > tj
                            lex_gr(args1, args2) && dominates_args()
                        } else {
                            false
                        }
                    }
                    // 定数は全ての関数適用より小さい
                    Int(_) | Str(_) => true,
                    _ => false,
                }
        }
        // 定数同士・定数と関数適用は比較不能／小さい
        (Int(_) | Str(_), _) => false,
        _ => {
            dbg!(&t1, &t2);
            unimplemented!();
        }
    }
}

fn lpo_gr_eq(t1: Rc<TermInner>, t2: Rc<TermInner>) -> bool {
    t1 == t2 || lpo_gr(t1, t2)
}

/// 変数vが項tに含まれるか
fn occur(v: &Var, t: &TermInner) -> bool {
    t.vars().contains(&v.clone())
}

/// LPO の辞書式拡張（同じ長さの引数列に対する厳密比較）
fn lex_gr(ts1: &[Rc<TermInner>], ts2: &[Rc<TermInner>]) -> bool {
    for (x, y) in ts1.iter().zip(ts2) {
        if x == y {
            continue;
        }
        return lpo_gr(x.clone(), y.clone());
    }
    false
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::util::{eq, opers, types};
    use crate::{
        completion::analyse::analyse,
        context_table::CtxtTable,
    };

    #[rstest]
    #[case("x: Int | p![o; x] = x")]
    #[case("x: Int | p![m!x x] = o;")]
    #[case("x y z: Int | p![x p![y z]] = p![p![x y] z]")]
    #[case("x y: Int | m!p![x y] = p![m!x m!y]")]
    fn test_analyse(#[case] input: &str) {
        let types = types(vec!["Int"]);
        let opers = opers(vec!["o", "p", "m"]);
        let ctxts = CtxtTable::new();

        let equation = eq(input, &types, &opers, &ctxts);
        dbg!(&equation);
        let rule = analyse(equation.context, equation.names, equation.left, equation.right);
        println!("{}", rule.unwrap());
    }
}
