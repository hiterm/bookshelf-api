## 1. Repository Batch Lookup

- [x] 1.1 Add the tenant-scoped batch-by-author method to `BookRepository`
- [x] 1.2 Implement one-query grouping in `PgBookRepository`
- [x] 1.3 Add repository tests for grouping, shared books, empty authors, and tenant isolation

## 2. Query Use Case

- [x] 2.1 Add and implement the batch book lookup on `QueryUseCase` and `QueryInteractor`
- [x] 2.2 Add use-case tests for batched IDs and DTO map conversion

## 3. GraphQL Resolution

- [x] 3.1 Add and test `BooksByAuthorLoader` with one batched use-case call
- [x] 3.2 Add the `Author.books` complex resolver and register its request-scoped DataLoader
- [x] 3.3 Add GraphQL tests for populated, empty, shared, and batched author book results

## 4. Verification

- [x] 4.1 Run formatting, Clippy, and the complete locked test suite
