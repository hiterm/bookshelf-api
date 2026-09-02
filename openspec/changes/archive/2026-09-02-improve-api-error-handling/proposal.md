## Why

Expected database conflicts are currently reported as infrastructure failures, bulk book updates can identify the wrong missing book, and GraphQL errors expose internal context without stable machine-readable categories. Clients need safe, actionable errors while operators retain enough internal detail to diagnose failures.

## What Changes

- Classify only known, operation-specific PostgreSQL constraint violations as domain conflicts, while preserving infrastructure fallback for unknown database failures.
- Make bulk book updates identify an actually missing or cross-tenant book ID within the existing transaction.
- Add stable GraphQL `extensions.code` values for not-found, validation, conflict, and internal failures through one presentation-layer conversion.
- Sanitize GraphQL messages so tenant identifiers, database details, and unexpected internal messages are not exposed, while retaining and logging the underlying errors.
- Add repository and presentation-boundary regression tests for the new classifications and public error contract.

## Capabilities

### New Capabilities

- `api-error-contract`: Defines context-aware persistence error classification and the safe, machine-readable GraphQL error contract.

### Modified Capabilities

- `bulk-book-update`: Requires bulk updates to report an actually missing or out-of-scope book ID rather than an arbitrary input ID.

## Impact

The change affects domain/use-case/presentation error conversion, PostgreSQL book and author repository mutation paths, GraphQL error serialization and logging, and related unit and database-backed repository tests. GraphQL error responses gain `extensions.code`; internal Rust error structures and existing transaction/event-recording boundaries remain intact.
