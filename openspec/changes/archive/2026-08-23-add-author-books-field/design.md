## Context

The GraphQL `Book` type already resolves its authors with a request-scoped `AuthorLoader`, but the inverse relationship is not exposed. Resolving books independently for each author would create N+1 database queries. The existing repository and query-use-case layers use concrete generic types and tenant-scoped identifiers.

## Goals / Non-Goals

**Goals:**

- Add `books` to the existing GraphQL `Author` type and return entries using the existing GraphQL `Book` type.
- Fetch relationships for all authors in one DataLoader batch and one database query.
- Preserve user isolation through every layer.
- Return a non-null empty list for authors without books.

**Non-Goals:**

- Defining book ordering or pagination.
- Introducing query complexity limits.
- Converting DataLoaders or query use cases to trait objects.
- Refactoring the existing generic architecture.

## Decisions

- Add a `BookRepository` batch method returning `HashMap<AuthorId, Vec<Book>>`. This follows the existing author batch API and keeps SQL concerns out of GraphQL. Per-author repository calls were rejected because they preserve N+1 behavior below the loader.
- Query `book_author` and `book` once for all requested author IDs, scoped by `user_id`, and group rows in memory. A book linked to multiple requested authors appears under every applicable key.
- Add a matching `QueryUseCase` method that parses string IDs and converts domain books to DTOs, preserving the current layer boundaries and generic dispatch.
- Register a request-scoped `BooksByAuthorLoader<QUC>` alongside `AuthorLoader<QUC>`. The loader value is `Vec<Book>`, so the resolver naturally exposes `[Book!]!` and maps missing keys to an empty list.
- Mark `Author` as a GraphQL complex object and resolve `books` through the loader. The existing `Book` object is reused unchanged.

## Risks / Trade-offs

- [Unspecified ordering can vary with query plans] → Tests compare membership rather than impose an order, and no `ORDER BY` contract is introduced.
- [Large author selections can produce a large batch result] → This change intentionally omits pagination; DataLoader still bounds database round trips to one query per batch.
- [Omitting zero-result keys could surface loader misses] → The resolver treats a missing loader value as an empty vector, while the precise internal key-population strategy remains an implementation detail.
- [Tenant data leakage through joins] → The batch query explicitly filters books by the authenticated `user_id`, and repository tests cover cross-user exclusion.
