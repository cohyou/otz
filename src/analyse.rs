use std::{cmp::Ordering, rc::Rc};

use crate::{context::Context, rule::Rule, symbol_table::Names, term::TermInner};

pub fn analyse(
    context: Rc<Context>,
    names: Rc<Names>,
    left: &Rc<TermInner>,
    right: &Rc<TermInner>,
) -> Option<Rule> {
    // println!("left: {:?} right: {:?}", left, right);

    analyse_inner(left, right).map(|cmp| {
        if cmp {
            // left > right
            Rule::new(context, names, left.clone(), right.clone())
        } else {
            // left < right
            Rule::new(context, names, right.clone(), left.clone())
        }
    })
}

fn analyse_inner(t1: &Rc<TermInner>, t2: &Rc<TermInner>) -> Option<bool> {
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
                        Ordering::Equal => analyse_inner(&args_f[0], &args_g[0]),
                    }
                }
                (TermInner::Int(_), TermInner::Fun(_, _)) => Some(false),
                // (TermInner::Var(_), TermInner::RuledVar(_, _, _)) => false,
                // (TermInner::RuledVar(_,_,_), TermInner::Var(_)) => true,
                _ => {
                    /*println!("t1: {:?} t2: {:?}", t1, t2);*/
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
mod tests {
    use combine::Parser;
    use rstest::*;

    use crate::{
        analyse::analyse,
        context_table::CtxtTable,
        id::{OperId, TypeId},
        parser::equation::equation_parser,
        symbol_table::SymbolTable,
    };

    #[rstest]
    #[case("x: Int | plus![0 x] = x")]
    #[case("x: Int | plus![minus!x x] = 0")]
    #[case("x: Int y: Int z: Int | plus![x plus![y z]] = plus![plus![x y] z]")]
    fn test_analyse(#[case] input: &str) {
        use std::rc::Rc;

        let types = SymbolTable::<TypeId>::new();
        types.assign("Int".to_string());
        let opers = SymbolTable::<OperId>::new();
        opers.assign("plus".to_string());
        opers.assign("minus".to_string());
        let ctxts = CtxtTable::new();
        ctxts.assign_to_current("x".to_string());

        let eq = equation_parser(&types, &opers, &ctxts)
            .parse(input)
            .unwrap()
            .0;
        dbg!(&eq);
        let mut names = ctxts.current_var_table();
        let oper_names = opers.current_table();
        names.extend(oper_names);
        let rule = analyse(eq.context, Rc::new(names), &eq.left, &eq.right);
        dbg!(&rule);
    }
}
