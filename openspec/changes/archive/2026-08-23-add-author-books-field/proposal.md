## Why

GraphQL clients cannot currently traverse from an author to that author's books. Adding this relationship allows clients to fetch author-centric views while batching database access to avoid N+1 queries.

## What Changes

- Add a non-null `books: [Book!]!` field to the GraphQL `Author` type.
- Return an empty list when an author has no books.
- Batch book lookup for multiple authors through the repository, query use case, and a request-scoped DataLoader.
- Reuse the existing GraphQL `Book` type without pagination or a specified ordering.

## Capabilities

### New Capabilities

- `author-books`: Covers querying books through GraphQL authors, empty relationships, tenant isolation, and batched resolution.

### Modified Capabilities

None.

## Impact

This affects the book repository interface and PostgreSQL implementation, query use-case interface and interactor, GraphQL loaders and objects, request context registration, and their unit and end-to-end tests. It does not change mutation behavior, pagination, ordering guarantees, or the existing generic use-case architecture.
