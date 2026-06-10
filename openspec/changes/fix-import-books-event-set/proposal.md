## Why

`import_books` GraphQL mutation で書籍を一括登録する際、複数の `create_with_event_set` 呼び出しにより `event_set` テーブルの `operation` カラムが `import_books` ではなく `create_author` または `create_book` で上書きされてしまう。これにより監査ログ（change history）の整合性が失われる。

また、E2E テストで `event_set.operation` 値を検証するための GraphQL API エンドポイントが存在しないため、テストでこの問題を検知できない。

## What Changes

- `ImportBooksInteractor::import` でトランザクション開始直後に `event_set` を `import_books` として `INSERT` する
  - 後続の `create_with_event_set` 内の `ON CONFLICT DO NOTHING` で既存行を検出し、operation は `import_books` のまま保持される
- `QueryUseCase` に `find_event_set_by_id` メソッドを追加する
- GraphQL Query に `eventSet(id: ID!): EventSet` リゾルバーを追加する
  - 認可：user_id で `event_set` テーブルをフィルタする
- E2E テストに `event_set.operation` 検証を追加する
  - `e2e_import_books` テストで `import_books` であることを確認
  - 他の change history テストでも `event_set.operation` 値を検証する

## Capabilities

### New Capabilities
- `event-set-query`: `eventSet` GraphQL クエリで `event_set` テーブルから単一レコードを取得する

### Modified Capabilities

## Impact

- `src/use_case/interactor/book.rs` — `ImportBooksInteractor::import` に `event_set` INSERT 追加
- `src/use_case/traits/query.rs` — `QueryUseCase` トレイトに `find_event_set_by_id` 追加
- `src/use_case/interactor/query.rs` — `QueryInteractor` で `find_event_set_by_id` 実装
- `src/use_case/dto/event.rs` — `EventSetDto` 追加
- `src/presentation/graphql/query.rs` — `eventSet` リゾルバー追加
- `src/presentation/graphql/object.rs` — `EventSet` GraphQL オブジェクト追加
- `e2e/tests/e2e.rs` — `event_set.operation` 検証追加
