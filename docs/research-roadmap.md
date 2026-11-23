# 研究ロードマップ・保留事項

> 目的: 議論した可能性を忘れずに残しつつ、MVP の実装を停止させない。ここにある候補は**採用済み実装仕様ではない**。

## R1. 等式推論と Lazy Saturation

議論してきた技術要素：TRS、Knuth–Bendix、Congruence Closure、e-graph、初期代数、CQL の saturation/chase、Curry の needed narrowing と fair search、Maude / Maude-SE、SAT/SMT。役割を混同しない。

| 技術 | 主な対象 |
| --- | --- |
| TRS / Completion / Congruence Closure / e-graph | 等式の正規化・同値判定 |
| Chase / saturation | 与えた理論や制約を満たす項・事実の生成／閉包 |
| Frozen query / relational matching | 状態中の適用条件・解の抽出 |
| Patch / rewriting | 世界の変更（Action） |
| SAT/SMT | 制約充足・補助的推論 |
| BFS / heuristic / folding | 状態空間探索・枝刈り |

**研究仮説**: インスタンス内部では必要な等式だけを `prove_I(t=u)` する。世界間の探索ではゴール関連の属性・アクションの前像だけを計算する。いずれも lazy と呼べるが、異なる階層の遅延計算であり、初期代数全体の saturation と同一視しない。

停止性、完全性、公平性、無限商に対する実装可能性が課題。必要な仕様を満たす段階で固定スキーマ上の有限実装へ戻る。

## R2. 論理・Doctrine・Hyperdoctrine

- 等式論理だけから出発する最小 elementary doctrine の構成。
- Context、型、等式、部分対象、subobject classifier の役割の区別。
- WHERE の述語と RETURN の射影を圏論的にどう組み合わせるか。
- 実存／全称と Beck–Chevalley、Frobenius などの成立条件。
- スキーマによって indexed な doctrine、schema change と論理式翻訳。
- $[C,\mathbf{Set}/W]$ 上の静的述語と span によるモダリティの統合。
- 型付きスキーマの構造を単一基底圏に無理に詰め込まない構想。

## R3. 記号的プランニングと高度な探索

- Powerlifted の lifted / powerlifted matching と比較。
- PDDL / STRIPS との対応。負条件や delete effect の semantics を正しく比較。
- State equivalence / isomorphism / bisimulation / subsumption。
- 前向き・後向き探索、計画書換え PbR、HTN、ヒューリスティクス。
- SAT/SMT/OMT による計画符号化。制約の伝搬をインスタンス側の等式処理とどう併用するか。
- 記号状態の有限表現、saturation と枝刈り。

## R4. データベース更新・圏論的書換え

- Entity Patch、Equation Patch と DPO/SqPO の対応条件。
- 同一性・削除・再生成を数学的にも実装的にもどう区別するか。
- DB morphism だけで更新が十分に表現できる場合とできない場合。
- スキーマ変更を含むアクションの圏論的意味論。
- typed attribute values と自由項の扱い。

## R5. システムの構造的・動的合成

- 共有インターフェース、スキーマ pushout、状態 pullback。
- Patch 更新の競合・同期、共有事象を伴う遷移系の pullback。
- インターフェース境界における制約保存。
- 同期／独立アクションとモノイダル構造、colored props / double categories。
- Assume–Guarantee、モデル検査、互換性・相互作用の保証。
- 並行性と causality を interleaving より詳細に扱う必要性。

## R6. モナドインスタンスと T-hyperdoctrine

Spivak の database monad instances、Maybe/Exception/非決定性/確率などに関連して、

$$
I:C\to Kl(M)
$$

のような Kleisli 値インスタンスを検討した。これは基本の集合値インスタンスからの拡張である。既存の $\Sigma_F\dashv\Delta_F\dashv\Pi_F$ が無条件にそのまま使えるとは限らない。

また、述語の値域を一般化する T-hyperdoctrine、量化・モダリティと組み合わせる可能性を探索した。しかし **MVP には取り込まない**。

## R7. 検証すべき疑問の優先順位

1. Frozen query の有限型付きマッチングは、実装上必要な Query を表現できるか。
2. Update 後状態がスキーマ制約を保つための検証をどう定義するか。
3. Patch の具体化は ID を保った意味で自然か。
4. 実行事象から span を構成した際、BFS の trace と一致するか。
5. 初期状態の同型／無限の状態生成に対して探索をどう管理するか。
6. 異なるスキーマ、内部論理、複合システムへの拡張が必要な用途を具体化できるか。

## スコープ管理

「美しい一般理論の証明」と「動く MVP」は別々に追跡する。研究テーマは関連する具体的な制約・反例・サンプル・参考論文を追加し、実装側に要求が生じた時点で Issue として切り出す。
