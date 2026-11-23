# Categorical Planning — 設計ドキュメント

旧プロジェクト名: **OTZ**。対象リポジトリ: [`cohyou/otz`](https://github.com/cohyou/otz)。この文書は2026-11-23までの対話から復元した**設計の引き継ぎ資料**。GitHub `main` ブランチのコードを部分的に確認したが、テスト実行・全API照合は未実施。

## 読む順番

1. [全体構想](overview.md) — 解決したい問題とレイヤー分離
2. [数学的モデル](mathematical-model.md) — インスタンス・frozen instance・クエリ
3. [最小アクション言語](action-language.md) — `Query + Patch` と実行アルゴリズム
4. [MVP とテスト](planning-mvp.md) — 先に動かす小さな目標
5. [既存実装の対応](current-implementation.md) — 2026-11-23時点のRustコード調査（部分）
6. [実装バックログ](implementation-backlog.md) — 実装順と完了条件
7. [圏論的意味論](categorical-semantics.md) — 実行事象・span・様相論理・動的貼り合わせ
8. [研究ロードマップ](research-roadmap.md) — 拡張の方向と保留事項
9. [参考資料](references.md) — 文献と議論の履歴

## 設計ステータスの読み方

- **採用方針**: 対話で現在のモデルとして採ることにした方向性。定理やコード上の完成を意味しない。
- **MVP 提案**: 実装を進めるために当面固定する仕様。既存コードの事情によって調整可能。
- **研究仮説**: 数学的妥当性や適用条件の検証が残るもの。
- **未解決**: 証明・仕様決定・実験のいずれかが必要。

## 基本原則

- **領域スキーマと遷移メタ構造を分離**。`State` や `Step` を Blocks World などの領域スキーマに無理に混ぜない。
- **等式推論・状態更新・探索・論理的検証を分離**。交換可能な実装アルゴリズムを選べるようにする。
- **理論の包括性よりも最小の縦切り実装を優先**。スキーマ固定、有限インスタンス、frozen-query のマッチ、原子的 Patch、探索から開始する。
- **数学的な説明と既存コードの対応は独立に検証**する。

## Architecture Decision Records

- [ADR-0001: 固定 type-side と MVP の状態](decisions/0001-fixed-type-side.md)
- [ADR-0002: Query + Patch アクション](decisions/0002-query-and-patch.md)
- [ADR-0003: 領域・遷移・論理レイヤーの分離](decisions/0003-separate-layers.md)
- [ADR-0004: インスタンスの等しさにスキーマの公理を含める](decisions/0004-equality-includes-schema-axioms.md)
- [ADR-0005: マッチの行き先は生成元に限り、全域で有限なインスタンスを前提とする](decisions/0005-match-targets-are-generators.md)
- [ADR-0006: LPO の優先順位では定数を最も小さくする](decisions/0006-lpo-precedence-constants-lowest.md)
- [ADR-0007: マッチの表現と列挙の方針](decisions/0007-match-representation.md)
