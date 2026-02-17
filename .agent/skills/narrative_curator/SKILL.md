---
name: narrative_curator
description: Standards for AI-generated narrative content and quality assurance.
---

# Narrative Curator Skill

## 1. Tone & Style
- **Standard Japanese**: 自然で、知的、かつ礼儀正しい標準語を使用する。
- **Polished Prose**: 小説や要約は、読者が没入できる質の高い文章を目指す。
- **No Sensationalism**: 「【緊急】」「〜の正体」などの煽りキーワードは一切禁止。

## 2. Domain Alignment
- VRChat体験の文脈を尊重し、単なるログではなく「想い出」としての価値を重視する。
- 専門用語やスラングは適切に扱い、文脈が不明な場合は無理に解釈せず客観的に記述する。

## 3. Quality Assurance
- 生成物のサイズチェック (空ファイルや極端に短いファイルの検知)。
- 重複表現や不自然な改行の排除。
- 複数の音声セッションを統合する際の論理性と時系列の正確性。

## 4. Iterative Improvement
- `task curator:eval` を通じたフィードバックループを回す。
- 低評価のコンテンツに対しては、プロンプトの調整や再生成を積極的に検討する。

## 5. Context7 Mastery (Layer 3, 4, 7)
- **Asset State**: Recordings と Summaries の 1:1 対応をセマンティックな正規状態として維持する。
- **Knowledge Sync**: 生成された物語が、プロジェクトのメタデータや README に記された世界観と矛盾しないよう監視する。
