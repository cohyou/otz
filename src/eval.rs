use std::{collections::HashMap, rc::Rc};

use crate::{
    completion::subst::{Subst, Var}, context::Context, equation::Equation, id::{OperId, TypeId, VarId}, instance::{Elem, Instance}, symbol_table::Names, term::{Term, TermInner}
};

/// A tableau over a schema S is a pair of:
/// • a context over S, called the for clause, fr and
/// • a set of quantifier-free equations between terms in Terms(S,fr), called the where clause wh.

/// An uber-flower S →T consists of, for each entity t ∈ T:
/// • a tableau (fr(t), wh(t)) over S and,
/// • for each attribute att: t→t′∈T, a term [att] in Termst′ (S,frt), called the return clause for att, and
/// • for each foreign key fk: t→t′∈T, a transform [fk] from the tableau for t′to the tableau for t (note
/// the reversed direction), called the keys clause for fk,
/// • such that an equality-preservation condition holds. We defer a description of this condition until
/// section 4.3.1.

#[derive(Default)]
pub struct Query(pub Vec<QueryEntity>);

#[derive(Default, Debug)]
pub struct QueryEntity {
    pub entity: Vec<TypeId>,
    pub fr: Context,
    pub wh: Vec<Equation>,
    pub ret: Vec<(OperId, TermInner)>,
    // keys: t -> t'
    // transform from tableau for t' to tableau for t
    pub keys: Vec<(OperId, VarId, TermInner)>,
}

pub fn eval(instance: Instance, query: Query) -> Instance {
    let saturated = instance.saturate();

    // generator
    let matches = query.0.iter().map(|query_entity| {
        query_entity.matches(&saturated)
    })
    // .inspect(|subst| { dbg!(&subst); })
    .collect::<Vec<_>>();

    let elems = matches.into_iter().flatten().map(|m| {
        Elem::Subst(m.into_subst())
    }).collect::<Vec<_>>();

    // #return・#keys の値の正規化にも、マッチの判定と同じ規則（データ + スキーマの公理）を使う
    let rules = saturated.equality_rules();

    // attの処理
    let attrs = query.0.iter().map(|qe| {
        qe.ret.iter().map(|(operid, term)| {
            // dbg!(operid, term);
            let attrs = elems.iter().filter_map(|e| {
                if let Elem::Subst(subst) = e {
                    let substed_term = term.substitute(subst);
                    let subst_vec = subst.sorted_entries();

                    let right_term = Term {
                        context: Rc::new(Context::default()),
                        names: Rc::new(Names::default()),
                        inner: substed_term.clone(),
                    };
                    let right = right_term.normalize(&rules);
                    Some(Equation {
                        context: Rc::new(Context::default()),
                        names: instance.names.clone(),
                        left: Rc::new(TermInner::Fun(
                            operid.clone(),
                            vec![
                                Rc::new(TermInner::Subst(subst_vec))
                            ],
                        )),
                        right: right.inner.clone(),
                    })
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            // dbg!(&attrs);
            attrs
        }).flatten().collect::<Vec<_>>()
    }).flatten().collect::<Vec<_>>();
    
    // foreign keyの処理
    let keys = query.0.iter().map(|qe| {
        qe.keys.iter().map(|(operid, varid, term)| {
            let keys = elems.iter().filter_map(|e| {
                if let Elem::Subst(subst) = e {
                    let substed_term = term.substitute(subst);
                    let subst_vec = subst.sorted_entries();

                    let keys_map = HashMap::from([(
                        Var::Id(varid.clone()),
                        substed_term.clone(),
                    )]);
                    let keys_subst = Subst::new(keys_map);
                    let right_subst = subst.compose(&keys_subst);
                    let right_subst_vec = right_subst.sorted_entries().into_iter().map(|(varid, inner)| {
                        let term = Term {
                            context: Rc::new(Context(HashMap::new())),
                            names: Rc::new(HashMap::new()),
                            inner: inner.clone(),
                        };
                        let inner = term.normalize(&rules).inner.clone();
                        (varid.clone(), inner)
                    }).collect::<Vec<_>>();

                    Some(Equation {
                        context: Rc::new(Context(HashMap::new())),
                        names: instance.names.clone(),
                        left: Rc::new(TermInner::Fun(
                            operid.clone(),
                            vec![
                                Rc::new(TermInner::Subst(subst_vec))
                            ],
                        )),
                        right: Rc::new(TermInner::Subst(right_subst_vec)),
                    })
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            keys
        }).flatten().collect::<Vec<_>>()
    }).flatten().collect::<Vec<_>>();
                    
    let data = [attrs, keys].concat();

    Instance {
        names: instance.names.clone(),
        schema: instance.schema,
        elems: elems,
        data: data,
    }
}

/// クエリのマッチ（証人）。frozen instance F_Q からインスタンス I への準同型
/// m: F_Q → I を、#for の各変数の行き先（I の要素）で表したもの。
/// 異なる変数が同じ要素に移ってもよい（非単射のマッチ）。
#[derive(Clone, Debug, PartialEq)]
pub struct Match(Subst);

impl Match {
    /// 変数 var の行き先の要素
    #[allow(dead_code)]
    pub fn get(&self, var: &VarId) -> Option<&Rc<TermInner>> {
        self.0.0.get(&Var::Id(var.clone()))
    }

    pub fn as_subst(&self) -> &Subst {
        &self.0
    }

    pub fn into_subst(self) -> Subst {
        self.0
    }
}

impl QueryEntity {
    /// この QueryEntity の tableau（#for と #where）のマッチを列挙する
    pub fn matches(&self, instance: &Instance) -> Vec<Match> {
        matches(instance, &self.fr, &self.wh)
    }
}

/// クエリ Q = (fr, wh) のインスタンス I におけるマッチ全体
///
///     Sol(Q, I) ≅ Hom(F_Q, I)
///
/// を列挙する。fr は型付き変数の context（#for）、wh は等式の連言（#where）。
///
/// - 変数の行き先は、I の生成元（#elem で宣言した要素）のうち型が合うもの。
/// - wh の各等式は、データの等式とスキーマの公理から導けるかで判定する
///   （Instance::equality_rules）。
/// - 結果の順序は決定的: 変数を VarId の昇順（#for で宣言した順）に並べ、
///   各変数の行き先を I.elems の順（#elem で宣言した順）に並べたときの辞書式順序。
///   VarId の小さい変数ほど上位の桁になる。
///
/// Uber-flower の評価では、マッチが eval(Q)(I)(t) の生成元になる:
/// eval(Q)(I)(t) := { [v_i → e_i] | I ⊢ eq[v_i → e_i], ∀eq ∈ wh(t), ∀e_i ∈ I_EA(s_i) }
pub fn matches(instance: &Instance, fr: &Context, wh: &[Equation]) -> Vec<Match> {
    // fr の各変数に、型の合う要素を独立に割り当てた環境（直積）をすべて作る。
    // Context は HashMap で反復順が実行ごとに変わるので、VarId 順に並べてから直積を作る。
    let mut vars = fr.0.iter().collect::<Vec<_>>();
    vars.sort_by_key(|(varid, _)| *varid);
    let substs = vars.into_iter().fold(
        vec![Subst::default()],
        |partials, (varid, tp)| {
            let mut next = Vec::new();
            for partial in &partials {
                for e in &instance.elems {
                    if let Elem::Oper(op) = e {
                        if op.cod.as_ref() == tp {
                            let mut m = partial.0.clone();
                            m.insert(
                                Var::Id(varid.clone()),
                                Rc::new(TermInner::Fun(op.id.clone(), vec![])),
                            );
                            next.push(Subst::new(m));
                        }
                    }
                }
            }
            next
        },
    );

    if wh.is_empty() {
        return substs.into_iter().map(Match).collect();
    }

    // 補完は一度だけ行い、すべての候補の判定で使い回す
    let rules = instance.equality_rules();

    // すべての等式を満たす必要がある(all)
    substs
        .into_iter()
        .filter(|subst| {
            wh.iter().all(|eq| {
                let left_substed = eq.left_term().substitute(subst);
                let right_substed = eq.right_term().substitute(subst);

                let substituted_equation = Equation {
                    context: left_substed.context,
                    names: left_substed.names,
                    left: left_substed.inner,
                    right: right_substed.inner,
                };
                // println!("substituted_equation: {}", substituted_equation);
                substituted_equation.holds_under(&rules)
            })
        })
        .map(Match)
        .collect()
}

impl TermInner {
    pub fn var(varid: VarId) -> Self {
        TermInner::Var(varid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oper::Oper;
    use crate::r#type::Type;

    fn make_oper_elem(id: usize, cod: &Type) -> Elem {
        Elem::Oper(Oper::new(
            OperId(id),
            Rc::new(Type::Unary(TypeId(0))),
            Rc::new(cod.clone()),
        ))
    }

    /// #for x y: Tp のように同じ型の複数変数がある場合、
    /// それぞれ独立に要素を割り当てた直積の環境がすべて得られること。
    ///
    /// 実際のクエリ構文では以下に対応する(#where等は無し):
    ///   #instance i
    ///   #for e1 e2: Emp
    /// Empが2件なら、(e1,e2)の組は (e1,e1)(e1,e2)(e2,e1)(e2,e2) の4通り。
    /// これはEmpテーブル自身とのCROSS JOIN(自己結合)に相当する。
    /// 修正前は1つの要素をe1,e2両方に割り当てていたため、
    /// (e1,e1)(e2,e2)のような対角成分しか得られず、
    /// (e1,e2)(e2,e1)のような異なる要素同士の組が漏れていた。
    #[test]
    fn test_eval_generators_cross_product_same_type() {
        let tp = Type::Unary(TypeId(1));
        let instance = Instance {
            names: Default::default(),
            schema: Default::default(),
            elems: vec![make_oper_elem(1, &tp), make_oper_elem(2, &tp)],
            data: vec![],
        };

        let mut fr = Context::default();
        fr.0.insert(VarId(0), tp.clone());
        fr.0.insert(VarId(1), tp.clone());

        let query_entity = QueryEntity {
            fr,
            ..Default::default()
        };

        let substs = query_entity.matches(&instance);
        // 2変数 x 2要素 = 4通りの直積になっているはず(修正前は2通りだった)
        assert_eq!(substs.len(), 4);
    }

    /// 型が異なる複数の#for行(エンティティ間のjoin)も、
    /// マージされた1つのcontextとして直積が取れること。
    ///
    /// 実際のクエリ構文では以下に対応する(#where等は無し):
    ///   #instance i
    ///   #for e: Emp
    ///   #for d: Dept
    /// Empが2件・Deptが1件なら、(e,d)の組は2通り。
    /// これはEmpとDeptのCROSS JOIN(全組み合わせ)に相当する
    /// (実務上は続けて `#where wrk!e = d` のように書いて
    ///  INNER JOIN相当に絞ることが多いが、このテストはその手前の
    ///  「直積が正しく生成されるか」だけを見ている)。
    /// 修正前は#for行ごとに別のcontextとして扱われ、
    /// eとdを両方含む環境が1件も生成されなかった。
    #[test]
    fn test_eval_generators_cross_product_different_types() {
        let tp1 = Type::Unary(TypeId(1));
        let tp2 = Type::Unary(TypeId(2));
        let instance = Instance {
            names: Default::default(),
            schema: Default::default(),
            elems: vec![
                make_oper_elem(1, &tp1),
                make_oper_elem(2, &tp1),
                make_oper_elem(3, &tp2),
            ],
            data: vec![],
        };

        let mut fr = Context::default();
        fr.0.insert(VarId(0), tp1.clone());
        fr.0.insert(VarId(1), tp2.clone());

        let query_entity = QueryEntity {
            fr,
            ..Default::default()
        };

        let substs = query_entity.matches(&instance);
        // tp1の要素2つ x tp2の要素1つ = 2通り
        assert_eq!(substs.len(), 2);
    }

    /// サンプルのインスタンス（example/instance/i.instance）とクエリを同じ記号表で読み込み、
    /// マッチを「変数↦要素」の文字列にして、matches が返した順に返す。
    /// 1 つのマッチの中も、変数を VarId 順（#for で宣言した順）に並べる。
    fn sample_matches(query: &str) -> Vec<String> {
        use crate::{context_table::CtxtTable, id::Symbol, symbol_table::SymbolTable};
        use combine::EasyParser;

        let types = SymbolTable::<TypeId>::new();
        let opers = SymbolTable::<OperId>::new();
        let ctxts = CtxtTable::new();
        let instance = crate::parser::parse_instance_with(
            "example/instance/i.instance",
            &types,
            &opers,
            &ctxts,
        );
        let query_entity = crate::parser::query::query_entity_parser::<combine::easy::Stream<&str>>(
            &types, &opers, &ctxts,
        )
        .easy_parse(query)
        .unwrap()
        .0;

        let var_names: HashMap<VarId, String> = ctxts
            .current_var_table()
            .into_iter()
            .filter_map(|(name, sym)| match sym {
                Symbol::Var(id) => Some((id, name)),
                _ => None,
            })
            .collect();
        let elem_name = |t: &TermInner| match t {
            TermInner::Fun(id, args) if args.is_empty() => opers
                .table
                .borrow()
                .iter()
                .find(|(_, v)| *v == id)
                .map(|(k, _)| k.clone())
                .unwrap(),
            other => panic!("element expected: {:?}", other),
        };

        let mut vars = query_entity.fr.0.keys().collect::<Vec<_>>();
        vars.sort();
        query_entity
            .matches(&instance.saturate())
            .iter()
            .map(|m| {
                vars.iter()
                    .map(|v| format!("{}↦{}", var_names[*v], elem_name(m.get(v).unwrap())))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect()
    }

    /// Hom(F_Q, I) に対応するマッチの列挙（バックログ C-03）
    ///
    /// サンプルのデータ（抜粋）:
    ///   wrk: e1↦d3, e2↦d2, e3↦d1, e4↦d2, e5↦d3, e6↦d2（e7 はデータなし）
    ///   mgr: e1↦e1, e2↦e4, e3↦e3, e4↦e4, e5↦e1, e6↦e7（e7 はデータなし）
    /// スキーマの公理:
    ///   mgr(mgr(e)) = mgr(e),  wrk(mgr(e)) = wrk(e)
    /// e7 にはデータが無いが、公理から次が導ける:
    ///   wrk(e7) = wrk(mgr(e6)) = wrk(e6) = d2
    ///   mgr(e7) = mgr(mgr(e6)) = mgr(e6) = e7
    #[rstest::rstest]
    // 成立する: 複数のマッチ。e7 はスキーマの公理を使って初めてマッチする
    #[case("#for e: Emp\n#where wrk!e = d2;", &["e↦e2", "e↦e4", "e↦e6", "e↦e7"])]
    // 成立しない: e2 を上司に持つ社員はいない
    #[case("#for e: Emp\n#where mgr!e = e2;", &[])]
    // 変数が複数: 異なる変数が同じ要素に移る非単射のマッチ（x↦e1,y↦e1 など）を含む
    #[case("#for x y: Emp\n#where mgr!x = y", &[
        "x↦e1,y↦e1", "x↦e2,y↦e4", "x↦e3,y↦e3", "x↦e4,y↦e4",
        "x↦e5,y↦e1", "x↦e6,y↦e7", "x↦e7,y↦e7",
    ])]
    // 外部キーを辿る: 途中の mgr の等式（Emp 型）も使う必要がある
    #[case("#for e: Emp\n#where wrk!mgr!e = d2;", &["e↦e2", "e↦e4", "e↦e6", "e↦e7"])]
    // 属性（Int / Str）の等式
    #[case("#for e: Emp\n#where sal!e = 250", &["e↦e1"])]
    #[case("#for e: Emp\n#where last!e = \"Gauss\"", &["e↦e1"])]
    // 型の異なる変数の結合
    #[case("#for e: Emp\n#for d: Dept\n#where wrk!e = d\n#where sec!d = e", &[
        "e↦e3,d↦d1", "e↦e5,d↦d3", "e↦e6,d↦d2",
    ])]
    // 条件なし: 型の合う全要素
    #[case("#for e: Emp", &["e↦e1", "e↦e2", "e↦e3", "e↦e4", "e↦e5", "e↦e6", "e↦e7"])]
    fn test_matches(#[case] query: &str, #[case] expected: &[&str]) {
        // 期待値は列挙順そのもの（ソートしない）
        assert_eq!(sample_matches(query), expected);
    }

    /// マッチの列挙順序は決定的（#15）。
    /// #for で先に宣言した変数（VarId が小さい）が上位の桁になり、各桁は #elem の宣言順に進む。
    #[rstest::rstest]
    // x が上位: x の宣言順（e1, e2, …）に並ぶ
    #[case("#for x y: Emp\n#where mgr!x = y", &[
        "x↦e1,y↦e1", "x↦e2,y↦e4", "x↦e3,y↦e3", "x↦e4,y↦e4",
        "x↦e5,y↦e1", "x↦e6,y↦e7", "x↦e7,y↦e7",
    ])]
    // 同じ条件で宣言順を入れ替えると y が上位になり、y の宣言順に並ぶ。
    // 名前のソート順とは異なる順序になることも確かめている
    #[case("#for y x: Emp\n#where mgr!x = y", &[
        "y↦e1,x↦e1", "y↦e1,x↦e5", "y↦e3,x↦e3", "y↦e4,x↦e2",
        "y↦e4,x↦e4", "y↦e7,x↦e6", "y↦e7,x↦e7",
    ])]
    fn test_matches_order(#[case] query: &str, #[case] expected: &[&str]) {
        // HashMap の反復順は HashMap ごとに変わり得るので、何度か読み直して同じ順序になることを確かめる
        for _ in 0..10 {
            assert_eq!(sample_matches(query), expected);
        }
    }
}
