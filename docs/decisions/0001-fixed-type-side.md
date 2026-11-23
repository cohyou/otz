# ADR-0001: 固定 type-side と固定スキーマで MVP を開始

- 状態: 採用方針
- 日付: 2026-11-23（2026-11-15 頃の議論を反映）

## Context

Algebraic Databases ではインスタンス、type-side の理論・代数、スキーマ変換を同時に扱える。更新と探索を初期段階から全て一般化すると検証しにくい。

## Decision

- 初期 MVP はスキーマ $S$ と具体 type-side 代数 $A$ を固定する。
- 世界 $w$ は $S\text{-Inst}_A$ の対象で表す。
- スキーマ自体を変更するアクションは後続課題とする。

## Consequences

- Query/Update の境界と整合性検証が明確になる。
- 任意の schema mapping と変化する type-side に対応する意味論は後から設計する。
- Frozen instance の記号項と具体 type-side の対応は別途仕様化する必要がある。
