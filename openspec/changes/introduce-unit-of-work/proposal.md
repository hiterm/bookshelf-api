## Why

各 `Pg*Repository` が個別に `pool.begin()` → `tx.commit()` を完結させているため、複数の集約リポジトリを同一トランザクションで束ねることが不可能です。その結果、`import_books` のような複合操作のために `PgImportBooksRepository` という暫定的な巨大リポジトリが生まれ、SQL重複や集約責務の混在といった負債が蓄積しています。Unit of Work（UoW）パターンを導入することで、トランザクション境界をリポジトリ実装の外に引き上げ、本来の集約リポジトリに責務を戻します。

## What Changes

- **New**: `PgUnitOfWork` 構造体をインフラ層に導入し、トランザクションの開始・コミット・ロールバックを一元管理します。
- **Modify**: 各 `Pg*Repository` の書き込みメソッド（`create`, `update`, `delete`, `restore`）を、pool 版（公開・トレイト実装用）と tx 版（crate 内限定・UoW 連携用）に切り分けます。
- **Remove**: `ImportBooksRepository` トレイトおよび `PgImportBooksRepository` を削除し、その責務を `PgBookRepository` + `PgAuthorRepository` + `PgUnitOfWork` に移譲します。
- **Modify**: `ImportBooksInteractor` を、`PgUnitOfWork` を使った複合トランザクションで再実装します。
- **Modify**: 各リポジトリの `#[cfg(test)]` DB 統合テストを、新しい tx 版メソッドでもカバーするよう調整します。

## Capabilities

### New Capabilities

- `unit-of-work`: インフラ層でのトランザクション共有機構。`sqlx::Transaction` のライフタイム管理とコミット・ロールバック制御を提供します。

### Modified Capabilities

- （該当なし。本変更は実装詳細の改善であり、ドメイン層の仕様や API 契約には変更ありません。）

## Impact

- **インフラ層**: `src/infrastructure/` に `PgUnitOfWork` を追加。各 `Pg*Repository` に `*_core` メソッドを追加。
- **ドメイン層**: 変更なし。既存トレイトを維持。
- **Use Case 層**: `ImportBooksInteractor` の実装が `PgUnitOfWork` 依存に変更。`ImportBooksRepository` 依存を削除。
- **テスト**: `ImportBooksRepository` のモックテストが不要に。`PgUnitOfWork` の統合テストを追加。
