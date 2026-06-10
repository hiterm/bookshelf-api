## Context

`ImportBooksInteractor::import` では `event_set` テーブルへの `INSERT` を行わず、各 `create_with_event_set` 呼び出しで `create_book`/`create_author` が `ON CONFLICT DO NOTHING` で上書き（最初のINSERTが勝つ）されるため、operation が `import_books` にならない。

また、E2E テストでは `event_set` テーブルの `operation` 値を確認する GraphQL API エンドポイントがなく、テストでこの問題を検知できない。

## Goals / Non-Goals

**Goals:**
- `import_books` 実行時に `event_set.operation` が `import_books` として記録されることを保証する
- E2E テストで `event_set.operation` 値を検証できる GraphQL API を提供する
- 他の change history テストでも `event_set.operation` 値を検証する

**Non-Goals:**
- `eventSet` から紐づくイベント一覧を取得する機能（将来的な機能拡張として保留）
- `event_set` テーブルのスキーマ変更
- `create_with_event_set` のインターフェース変更

## Decisions

**1. 最小変更: `sqlx::query` をユースケース層で直接使用**
- `ImportBooksInteractor::import` 内で `event_set` INSERT を直接実行する
- 理由: `create_with_event_set` のトレイトインターフェースを変更しない最小変更。リポジトリの責務は「指定された `event_set_id` に対してエンティティのイベントを記録する」であり、operation 値の決定はユースケース層の責務と考える
- 代替案: `create_with_event_set` に `operation` 引数を追加 → リポジトリ実装すべてに変更が必要なため見送り

**2. `eventSet` クエリの実装: `QueryInteractor` 内で `sqlx::query` 直接使用**
- `event_set` テーブルの読み取りは単純な SELECT なため、専用リポジトリは作成しない
- 理由: `QueryInteractor` は既存クエリで `sqlx::query` を直接使用している（例: `find_book_by_id`）

**3. 認可: SQL レベルで `user_id` フィルタ**
- `WHERE id = $1 AND user_id = $2` で他ユーザーの event_set へのアクセスを防止
- 理由: `event_set` テーブルに `user_id` カラムがあるため、SQL レベルで簡単に実現可能

## Risks / Trade-offs

- [sqlx::query をユースケース層で直接使用] → テストではモック化できないため、統合テストで検証する必要がある
  - 緩和: `#[sqlx::test]` 統合テストで `event_set.operation` を直接検証

- [eventSet クエリに対応する専用リポジトリがない] → 将来的に event_set 関連のクエリが増えた場合、リポジトリの追加を検討
  - 緩和: 現時点では1クエリのみのため、簡潔に保つ

## Migration Plan

- データベースマイグレーション不要（既存スキーマで対応）
- デプロイ: 通常のアプリケーションデプロイ
- ロールバック: 前のバージョンに切り替え
