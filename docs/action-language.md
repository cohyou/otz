# 最小アクション言語：Frozen Query + Entity/Equation Patch

> ステータス: 2026-11-21の議論を中心とする**MVP 提案仕様**。コード上の構文、エラー型、永続 ID の具体形式は未確定。

## 1. Action の形

$$
\operatorname{Action}\;a=(Q_a,U_a)
$$

- $Q_a$ は適用候補を列挙する frozen-instance query。
- $U_a$ はマッチに依存する記号的更新規則。
- $m\in\operatorname{Hom}(F_{Q_a},I)$ から Symbolic Patch を具体化する。
- 検証が成功した場合だけ、新しい状態 $J$ を得る。

```text
Query -> Matches -> Symbolic Patch -> Concrete Patch
                                      -> Validate -> Apply -> Next State
```

## 2. 更新の最小操作

| 操作 | 効果 | 注意 |
| --- | --- | --- |
| `Create(entityType, freshId, fields)` | 新しいエンティティを生成 | ID は毎回新規、必要な全域関数値を確定 |
| `Delete(entityId)` | 対象エンティティを削除 | 参照元がある場合の dangling を検査 |
| `Set(entityId, field, value)` | 属性または外部キーの値を置換 | **等式の単純な追加とは異なる** |

**Entity Patch** は生成・削除、**Equation Patch** は属性・外部キーの関数値の設定／変更を担当する。将来は一般的なインスタンス等式の追加・削除を研究できるが、MVP の `Set` と同一視しない。

## 3. 記号的な構文例

以下は概念例であり、既存パーサーの構文ではない。

```text
action move(x: Block, from: Location, to: Location):
  match:
    x.loc = from
    to.isAvailable = true
  patch:
    Set(x, loc, to)
```

属性 `isAvailable` を Bool 型に含めるか、他の等式・別エンティティによって表現するかは Blocks World のスキーマ定義次第。禁止条件（`not`、不存在判定）を最初から暗黙に導入しない。

## 4. 実行の形式化

マッチ集合

$$
M_a(I)=\operatorname{Hom}(F_{Q_a},I)
$$

に対して、マッチから記号的更新を作る写像を考える：

$$
\operatorname{instantiate}_{a,I}:M_a(I)\to\operatorname{SPatch}(I)
$$

`prepare` は新規 ID を確保し、マッチの変数を既存 ID と値に解決して Concrete Patch を作る。`validate` は更新後に成立すべき整合性を、実行前に検証する。`apply` は**同時・原子的**に実施する。

```text
execute(a, I):
  for m in matchQuery(a.query, I):
    symbolic = instantiate(a.update, m)
    concrete = prepare(symbolic, I, freshIds)
    if validate(concrete, I):
      yield apply(concrete, I)
```

API 名や引数は設計候補であり、正確なシグネチャは別途決める。

## 5. 整合性のチェック

- スキーマの等式公理を更新後状態が満たす。
- 各属性・外部キーが型に合致する。
- 全域関数に値の欠落や複数値が生じない。
- ID が重複せず、削除済み ID を再利用しない。
- 削除後に dangling reference が残らない。
- 競合する `Set`、`Set` と `Delete` の同時操作を解決する（まずは拒否でもよい）。
- エンティティ同士の等式による同一視がある場合、ID 単位の更新と商の関係が定義されている。

**適用前提の不成立** と **Patch 適用時の整合性違反** は別種の失敗とする。

## 6. 同一性と状態重複

実装上の entity ID は削除・再生成を区別するために永続化する。しかし探索の状態重複を ID 完全一致で判定すると、数学的に同型な状態まで別状態として扱う可能性がある。

初期 MVP は保守的な構造的ハッシュ＋明示された同値判定を使い、ID 無視の同型判定・正規化は独立した最適化課題として扱う。ID を無視してよい場合といけない場合の定義は未決。

## 7. 自然性・圏論的書換えとの関係（研究課題）

マッチ $m$ から Patch への変換は、インスタンス同型による ID の付け替えに対して整合的であるべき。ただし、任意の準同型に対して適用可能性が保存されるとは限らない（削除やマージを含むため）。

Entity/Equation Patch が DPO、SqPO、部分写像、span のどれに対応するかは対象のインスタンス圏・モノ射・存在条件を明示して検証する。**Patch を DPO と定義済みだとは主張しない**。

## 8. 次に決めること

1. Concrete Instance のシリアライズ形式と ID 型。
2. `Create` が必要な外部キー値をどこで設定するか。
3. 同時 Set / Delete の衝突判定と実行結果型。
4. インスタンス同型をいつ使い、どの識別子を保持するか。
5. Query matcher の遅延実装と fair enumeration の条件。
