# 既存コードの現状（2026-11-23 部分調査）

> ステータス: GitHub `cohyou/otz` の **main ブランチを読んだ結果**。2026-11-23 にローカルで `cargo test` を実行し、全テストが通ることを確認した（下記「テストの状態」）。ただし全ファイルの精査はしておらず、新設計の完成度を断定しない。

## リポジトリ

- `Cargo.toml`: Rust edition 2021、`combine` 4.6.7、`autoincrement` 1.0.1、テスト用 `rstest` 0.26.1。
- `README.md`: 短い OTZ / algebraic database by rust の説明のみ。
- `example/schema/`, `example/instance/`, `example/query/`, `example/theory/`: 現行DSLの具体例。
- `src/main.rs`: `parse_instance`、クエリファイルのパース、`eval` の実行と出力という試行経路がある。
- `src/schema.rs`: `Schema { theory, entities, fkeys, attrs, constraints }`。
- `src/instance.rs`: `Instance { names, schema, elems, data }`。`elems` は `Elem::Oper` / `Elem::Subst`、`data` は `Vec<Equation>`。
- `src/eval.rs`: `Query(Vec<QueryEntity>)`、`QueryEntity { entity, fr, wh, ret, keys }`、`eval(instance, query) -> Instance`。インスタンスの `saturate()` を呼び、マッチを列挙して出力インスタンスを構築している。マッチの列挙は `matches(instance, fr, wh) -> Vec<Match>`（`QueryEntity::matches`）として独立した API になっている。
- `src/parser/query/`: `#for` / `#where` / `#return` / `#keys` に関するパーサー。
- `src/completion/`, `src/reduct.rs`, `src/saturate.rs`: 等式補完、項書換え、飽和に関連する実装。

## 重要な差分

**アクションの Query は、`eval` による Uber-flower 型のクエリ評価と同一の API ではない。** `eval` は出力 `Instance` を返すが、アクション実行には個別のマッチ（証人）が必要で、それを Patch に引き渡す。マッチは `eval::matches` で直接取り出せる（バックログ C-03）。

具体的な差分：

1. `QueryEntity.fr` は単一の `Context`。PR #1（マージ済み）で、`#for` 句の複数変数・複数エンティティを直積として列挙するようになった。
2. `eval::matches` は、`#for` の各変数の行き先を `Instance.elems`（`#elem` で宣言した生成元）から選ぶ。`#where` の等式は、データの等式とスキーマの公理をまとめて補完した規則（`Instance::equality_rules`）で判定する。サンプルのインスタンスで、成立する／しない、複数のマッチ、非単射のマッチ、外部キーを辿る等式、属性の等式、結合をテストしている。生成元ではない項（データが無い要素の `wrk!e7` など）は、マッチの行き先にならない。type-side の値（`Int` 型の変数など）を行き先にするマッチは未対応。
3. 等式の判定（`Instance::equality_rules`、`Instance::deducible`）は、データの等式とスキーマの公理（`constraints`）の両方を使う。`Equation::to_rule`（向きを決めずに規則にする）は使われなくなった。Knuth–Bendix 補完（`completion::complete`）の規則の向き付けは LPO（`completion/analyse.rs`）で行う。LPO で比較できない等式（可換律など）は規則にせず保留するため、その場合の補完結果は不完全になる（停止性を優先し、健全性は保つ）。LPO の優先順位では、定数（0 引数の記号）を全ての関数記号より小さくしている。そのため `wrk(e2) = d2` は `wrk(e2) → d2` の向きになり、正規形はできるだけインスタンスの要素になる。
4. `src/instance.rs` の構造には、独立した永続 Entity ID と原子的 Entity/Equation Patch API は見えていない（ただし別ファイルや既存機能に存在しないと断定はしない）。
5. `src/main.rs` は実行例の入口で、計画探索のエンドツーエンド経路はこの範囲では確認できなかった。

## テストの状態（2026-11-23）

`cargo test`: 94 件すべて成功。以前は 6 件の失敗に加え、`completion::tests::test_complete` のスタックオーバーフローでテストが途中で止まっていた。修正した内容は次のとおり。

- LPO の誤りを修正した。変数に対する比較の否定が逆になっていたことと、辞書式比較が構造的な `Ord` を使っていたこと。この誤りのため `x → p(x, o)` のような項を大きくする規則が生まれ、正規化が発散していた。
- 群の公理から、よく知られた 10 本の完備な書換え系が得られることをテストで確認するようにした（3 公理から始める古典例を含む）。
- `example/` へ移動したファイルのパスが、テスト側で古いままだったのを修正した。
- テストデータの誤りを修正した（文脈の欠落、マッチしない例）。

多くのテストは `dbg!` や `println!` で出力するだけで、アサーションを持たない。回帰テストとして機能させるには、期待値を書き足していく必要がある。

## 次の作業

- type-side の値を行き先にするマッチと、非単射のマッチを禁止したい場合の扱いを決める（C-04）。
- `Match` を Symbolic Patch に引き渡す（D-01、D-02）。
- `saturate()` は現状インスタンスを複製するだけなので、スキーマの公理による飽和との関係を整理する。
