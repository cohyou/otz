# 既存コードの現状（2026-11-23 部分調査）

> ステータス: GitHub `cohyou/otz` の **main ブランチを読んだ結果**。全ファイル・全テストの検証、ローカルビルドはまだ行っていない。新設計の完成度を断定しない。

## リポジトリ

- `Cargo.toml`: Rust edition 2021、`combine` 4.6.7、`autoincrement` 1.0.1、テスト用 `rstest` 0.26.1。
- `README.md`: 短い OTZ / algebraic database by rust の説明のみ。
- `example/schema/`, `example/instance/`, `example/query/`, `example/theory/`: 現行DSLの具体例。
- `src/main.rs`: `parse_instance`、クエリファイルのパース、`eval` の実行と出力という試行経路がある。
- `src/schema.rs`: `Schema { theory, entities, fkeys, attrs, constraints }`。
- `src/instance.rs`: `Instance { names, schema, elems, data }`。`elems` は `Elem::Oper` / `Elem::Subst`、`data` は `Vec<Equation>`。
- `src/eval.rs`: `Query(Vec<QueryEntity>)`、`QueryEntity { entity, fr, wh, ret, keys }`、`eval(instance, query) -> Instance`。インスタンスの `saturate()` を呼び、クエリ環境を列挙して出力インスタンスを構築している。
- `src/parser/query/`: `#for` / `#where` / `#return` / `#keys` に関するパーサー。
- `src/completion/`, `src/reduct.rs`, `src/saturate.rs`: 等式補完、項書換え、飽和に関連する実装。

## 重要な差分

**今回のアクション Query は、現行の `eval` による Uber-flower 型のクエリ評価と同一の API ではない。** 現行 `eval` は出力 `Instance` を返す。提案中のアクション実行は個別のマッチ（証人）を取り出して、それを Patch に引き渡す必要がある。既存構造を再利用できるかを先に検証する。

具体的な差分：

1. `QueryEntity.fr` は `Vec<Context>`（`main`）。複数変数を正しく列挙するための PR #1 が開いている（2026-10-25）。その PR では単一 `Context` に統合して直積を生成する方針。**未マージのため main の仕様と混同しない。**
2. `src/eval.rs` の `eval_generators` は `Instance.elems` の要素を使う実装。数学的な `Hom(F_Q,I)` との一致、複数マッチ・非単射マッチ・型理論の評価をテストする必要がある。
3. `src/reduct.rs` に `TODO: 本来はSchemaのconstraintsも必要`、`Equation::to_rule` に規則の向き決定が必要との TODO がある。MVP の検証でこの前提に依存する場合は要対応。
4. `src/instance.rs` の構造には、独立した永続 Entity ID と原子的 Entity/Equation Patch API は見えていない（ただし別ファイルや既存機能に存在しないと断定はしない）。
5. `src/main.rs` は実行例の入口で、計画探索のエンドツーエンド経路はこの範囲では確認できなかった。

## 既存のオープン PR

- [#1 fix: #for句の複数変数・複数エンティティを直積で評価](https://github.com/cohyou/otz/pull/1)
- PR の説明によれば、追加テストを実施し、修正前から存在する一部失敗は残っている。**当文書では CI/テストを再実行していない**。

## 推奨する最初の作業

- PR #1 の内容とテストを確認し、採用するか判断する。
- 既存の評価結果の `Instance` と「マッチ証人」の抽出を比較する小テストを作る。
- type-side・スキーマ制約と既存の `saturate` の関係を確認する。
- そのうえで `Query → Match → Patch` の最初の一歩を現行構造に載せる。
