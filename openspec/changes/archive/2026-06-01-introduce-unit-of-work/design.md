## Context

各 `Pg*Repository`（`PgBookRepository`, `PgAuthorRepository`, `PgUserRepository` など）は `PgPool` を直接保持し、書き込みメソッド（`create`, `update`, `delete`, `restore`）の内部で `pool.begin()` → `tx.commit()` を完結させています。この設計により、1つの Use Case が複数の集約リポジトリを呼び出しても、それぞれが別々のトランザクションになります。

`import_books` のような複合操作では、Author の upsert → Book の insert → event 記録 を同一トランザクションでアトミックに行いたいため、暫定的に `PgImportBooksRepository` が作られました。これは `book`, `author`, `event_set`, `book_event`, `author_event` すべてに直接アクセスし、SQL が `PgBookRepository` / `PgAuthorRepository` と重複しています。

Unit of Work（UoW）パターンを導入し、トランザクション境界をリポジトリ実装の外に引き上げることで、この負債を解消します。

## Goals / Non-Goals

**Goals:**
- インフラ層に `PgUnitOfWork` を導入し、複数リポジトリ間でトランザクションを共有できるようにする
- 各 `Pg*Repository` の書き込みメソッドを、pool 版と tx 版に切り分ける
- `ImportBooksRepository` / `PgImportBooksRepository` を削除し、責務を元の集約リポジトリに戻す
- イベント記録を含む複合操作を完全にロールバック可能にする

**Non-Goals:**
- ドメイン層トレイト（`BookRepository`, `AuthorRepository` など）の変更
- API スキーマや GraphQL スキーマの変更
- 新たな外部依存の導入
- `sqlx` 以外の ORM/DB ライブラリへの移行

## Decisions

### Decision 1: `PgUnitOfWork` は `sqlx::Transaction` の薄いラッパーとする

**Rationale:** `sqlx::Transaction` のライフタイム制約（`'static` が必要な場面など）は Rust の async 境界で扱いにくくなりがちですが、UoW はインフラ層内に閉じるため、直接的に `sqlx::Transaction<'a, Postgres>` をラップするだけで十分です。過度な抽象化は避け、必要最小限のインターフェース（`begin`, `commit`, `rollback`, `tx`）を提供します。

**Alternatives considered:**
- `Arc<Mutex<Transaction<'static, Postgres>>>` をリポジトリに渡す → 不要なロックオーバーヘッドとデッドリスク
- ドメイントレイトに `Connection` 抽象を追加する → `async-trait` + `sqlx::Executor` のライフタイムが Rust 2024 edition でも複雑になり、既存テストへの影響が大きい

### Decision 2: 書き込みメソッドを pool 版と tx 版に分離する

各 `Pg*Repository` の書き込みメソッドは以下の構成にします：
- **公開メソッド**（トレイト実装用）: `pool.begin()` → `*_core` → `tx.commit()`
- **内部メソッド** `pub(in crate::infrastructure)`: `*_core(tx, ...)` — 実際の SQL 実行とイベント記録

**Rationale:** ドメイントレイトを変更せずに済む。`PgUnitOfWork` は内部メソッドを呼び出す。`find_by_id`, `find_all` などの読み取りメソッドは変更不要（pool 参照で十分）。

**Alternatives considered:**
- 全メソッドを `Executor` 汎化する → `sqlx::query` の `.execute(&mut **tx)` と `.execute(&pool)` の互換性はあるが、トレイト境界が複雑化
- リポジトリ自体を tx 構築時に生成する → リポジトリのライフタイムが UoW に束縛され、既存の DI 構成（`Arc<dyn BookRepository>` など）に影響

### Decision 3: `ImportBooksInteractor` は `PgUnitOfWork` を直接使用せず、新たな `ImportBooksService`（インフラ層）を経由する

**Rationale:** `ImportBooksInteractor` は Use Case 層に属し、インフラ層の `sqlx::Transaction` に直接依存すべきではありません（レイヤー違反）。しかし `PgUnitOfWork` は sqlx 特有の型を含むためドメイン層には置けません。そこで、インフラ層に `ImportBooksService`（または `PgImportBooksService`）を置き、`PgUnitOfWork` + `PgBookRepository` + `PgAuthorRepository` を使って `import` 処理を実装します。`ImportBooksInteractor` はこのサービスのトレイト（ドメイン層 or Use Case 層に定義）に依存します。

**Alternatives considered:**
- `ImportBooksInteractor` 内で直接 `PgUnitOfWork` を使う → Clean Architecture 的にはレイヤー違反
- `BookRepository` / `AuthorRepository` トレイトに tx 対応メソッドを追加する → ドメイン層が `sqlx` に依存することになる

## Risks / Trade-offs

- **[Risk]** `*_core` メソッドの増加により、リポジトリ実装が少し冗長になる  
  → **Mitigation**: `macro_rules!` やヘルパー関数で pool 版のボイラープレートを共通化する。読み取りメソッドは変更しないため影響は限定的。

- **[Risk]** `PgUnitOfWork` のライフタイム `'a` が複数のリポジトリ呼び出しで複雑になる  
  → **Mitigation**: `&mut uow` を順次渡すパターンに統一。`uow.tx()` の戻り型は `&mut Transaction<'_, Postgres>` で、所有権移動は生じない。

- **[Risk]** `ImportBooksRepository` の削除により、既存の `MockImportBooksRepository` を使ったテストが失われる  
  → **Mitigation**: `ImportBooksService` トレイトに `automock` を付与し、`ImportBooksInteractor` のユニットテストを継続。DB 統合テストは `PgUnitOfWork` + `Pg*Repository` の組み合わせで実施。

## Migration Plan

1. `PgUnitOfWork` を実装（インフラ層に追加）
2. `PgBookRepository` / `PgAuthorRepository` / `PgUserRepository` / その他書き込みリポジトリに `*_core` メソッドを追加
3. `ImportBooksService` トレイト（ドメイン層 or Use Case 層）と `PgImportBooksService`（インフラ層）を新規作成
4. `PgImportBooksRepository` と `ImportBooksRepository` トレイトを削除
5. `ImportBooksInteractor` を `ImportBooksService` 依存に変更
6. 各種テストを調整・追加
7. pre-commit チェック（`cargo fmt`, `cargo clippy`, `cargo test`）を実行

## Open Questions

- `PgUnitOfWork` の `tx` メソッドが `&mut Transaction` を返す設計で、複数リポジトリ同時呼び出し（並列ではなく直列）に支障がないか、実装時に検証が必要
- `ImportBooksService` のトレイト定義場所は Use Case 層（`use_case::traits`）とするか、それとも新たなドメインサービス層を設けるか
- 将来的に `RestoreBookUseCase` / `RestoreAuthorUseCase` も複数テーブルを跨ぐ操作を行うため、UoW 対応が必要かどうか
