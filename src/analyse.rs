// use core::panic;
use std::{cmp::Ordering, rc::Rc};

use crate::{context::Context, rule::Rule, subst::Var, symbol_table::Names, term::{TermInner}};

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
            &TermInner::Subst(_, inner) => inner.size(),
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
            &TermInner::Subst(_, inner) => inner.var_size(),
        }
    }
}

fn lpo_gr(t1: Rc<TermInner>, t2: Rc<TermInner>) -> bool {
    lpo_gr_eq(t1.clone(), t2.clone()) && !lpo_gr_eq(t2.clone(), t1.clone())
}

fn lpo_gr_eq(t1: Rc<TermInner>, t2: Rc<TermInner>) -> bool {
    // println!("lpo_gr_eq: t1: {:?}, t2: {:?}", t1, t2);
    match (t1.as_ref(), t2.as_ref()) {
        (t, TermInner::Var(xi)) => occur(&Var::Id(xi.clone()), t),
        (t, TermInner::RuledVar(xi, rid, kind)) => occur(&Var::Ruled(xi.clone(), *rid, kind.clone()), t),
        (TermInner::Var(_), _) | (TermInner::RuledVar(_, _, _), _) => false,
        (TermInner::Fun(f1, args1), TermInner::Fun(f2, args2)) => {
            (
                f1 == f2 &&
                lex_gr_eq(lpo_gr_eq, args1.clone(), args2.clone()) &&
                args2.iter().all(|arg2| lpo_gr_eq(t1.clone(), arg2.clone()) )
            ) ||
            (
                f1 > f2 &&
                args2.iter().all(|arg2| lpo_gr(t1.clone(), arg2.clone()))
            ) ||
            args1.iter().any(|arg1| lpo_gr_eq(arg1.clone(), t2.clone()))
        }
        _ => unimplemented!()
    }
}

/// 変数vが項tに含まれるか
fn occur(v: &Var, t: &TermInner) -> bool {
    t.vars().contains(&v.clone())
}

fn lex_gr_eq(gr_eq: fn(Rc<TermInner>, Rc<TermInner>) -> bool, ts1: Vec<Rc<TermInner>>, ts2: Vec<Rc<TermInner>>) -> bool {
    match ts1.cmp(&ts2) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => {
            for (x, y) in ts1.iter().zip(&ts2) {
                if gr_eq(x.clone(), y.clone()) && !gr_eq(y.clone(), x.clone()) {
                    return true;
                } else if !(gr_eq(x.clone(), y.clone()) && gr_eq(y.clone(), x.clone())) {
                    return false;
                }
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::util::{eq, opers, types};
    use crate::{
        analyse::analyse,
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
