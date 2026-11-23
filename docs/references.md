# 参考資料・議論の来歴

この文書は **2026-11-23 時点までのユーザーと ChatGPT の議論を整理した設計メモ**。論文の引用箇所・定理番号と設計判断の対応を厳密に証明した文書ではない。以下の URL は議論で挙がった原典・資料への入口で、各文献の正確な適用範囲は読む際に再検証する。

## 以前から参照した資料

- [OTZ repository](https://github.com/cohyou/otz) — 対象リポジトリ（`main` の主要モジュールのみ部分確認）
- [Algebraic Databases に関する arXiv:1602.03501](https://arxiv.org/abs/1602.03501)
- [Algebraic Databases 関連 arXiv:1503.03571](https://arxiv.org/abs/1503.03571)
- [関連 arXiv:2501.02413](https://arxiv.org/abs/2501.02413)
- [関連 arXiv:1803.05316](https://arxiv.org/abs/1803.05316) — 特に章 1,2,4,5,6,7 のモデル解釈を検討
- [Monad instances に関する arXiv:1209.1011](https://arxiv.org/abs/1209.1011)
- [Category Theory の公開テキスト（第7章の Monad in Databases に言及）](https://ia801604.us.archive.org/7/items/cattheory/cattheory.pdf)
- [Emily Riehl, Category Theory in Context](https://emilyriehl.github.io/files/context.pdf)
- [GitHub: 数式の記述](https://docs.github.com/ja/get-started/writing-on-github/working-with-advanced-formatting/writing-mathematical-expressions)
- [GitHub: Mermaid diagrams](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-diagrams)

## 検討した関連概念・システム

CQL / Algebraic Databases、Schema / Instance / Data Migration、Frozen Instance、Powerlifted、PDDL / STRIPS、Blocks World / Logistics、SAT/SMT、TRS、Congruence Closure、e-graphs、Chase、Maude / Maude-SE、Curry / Needed Narrowing、DPO / SqPO、Span / Cospan、Doctrine / Hyperdoctrine、[C, Set/W]、Dynamic Logic、Petri Nets、Open Petri Nets、HTN、PbR、Monad Instances、T-Hyperdoctrine。

## 主な対話記録（タイトル・日付）

利用可能な会話要約から復元。個別会話 URL は付与しない。

- 2026-11-07: 状態族 $[C,\mathbf{Set}/W]$ とアクション span、領域スキーマとの分離。
- 2026-11-08: DPO 規則、遷移 span、内部論理との接続。書換え・推論技術の役割分担。
- 2026-11-15: 固定 type-side、frozen instance の世界別解集合、action matching から span の構成。
- 2026-11-17: 最小等式論理、frozen instance、Doctrine のスキーマ依存解釈、アクションに関する総整理。
- 2026-11-20: システム設計・モデル検査、複数システムの静的・動的貼り合わせ、共有インターフェース。
- 2026-11-21: 最小アクション言語、Entity/Equation Patch、永続 ID、更新・整合性の詳細。Monad instances と T-hyperdoctrine。
- 2026-11-23: GitHub 実装管理への移行・設計の保存。

## 古いメモ

Google Drive「メモ - 『otz』」の参照先は `https://docs.google.com/document/d/1Mu3Pr-9toeDkZYEzIBolaPhQJCrDbiCp1dU9r38KLn8/edit`。以前は `Query` / `Eval` の実装途中だった記憶、Java の既存実装を読める可能性、圏論ベースのシステム記述言語・推論・プランニングという初期問題意識が記載されている。**いずれも現行ソースコードの確認前の記憶・メモ**として扱う。

## provenance の制限

すべての過去のチャットの逐語的な全文を取り込んだものではない。主要な技術議論とユーザーの継続的構想から設計を再構成し、未検証部分に明示的なラベルを付けた。元の会話や論文を後から参照して訂正する可能性がある。
