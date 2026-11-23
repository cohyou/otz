# ADR-0002: Action は Query + Entity/Equation Patch

- 状態: MVP 提案仕様
- 日付: 2026-11-23（2026-11-21 の設計を反映）

## Context

マッチするエンティティを特定してその値を変更するには、クエリの真偽だけでなく解の証人が必要。単に等式を追加する処理では、既存関数値の置換にならない。

## Decision

- Action の適用対象は frozen-instance query によるマッチで決める。
- 更新は `Create/Delete/Set` の Symbolic Patch から生成する。
- `Set` は関数値の**置換**。スキーマ等式と混同しない。
- Concrete Patch を同時・原子的に適用し、事前に整合性を検証する。
- entity ID は永続で、削除後に再利用しない。

## Consequences

- 実行の各結果に match と Patch の証人を保持できる。
- 一般の等式追加・削除、DPO/SqPO との対応は別研究。
- 削除時の dangling、同時 Set の衝突、quotient 内での同一性等を明示的に扱う必要がある。
