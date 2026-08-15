## 1. Repository and use-case results

- [x] 1.1 Return inserted event IDs from Book create/update repository operations and cover them with unit tests
- [x] 1.2 Return inserted event IDs from Author create/update repository operations and cover them with unit tests
- [x] 1.3 Add `SingleEventMutationResultDto<T>` and migrate only Book/Author create/update use-case contracts
- [x] 1.4 Verify create/update transaction failure paths cannot return an event ID
- [x] 1.5 Represent returned create/update event IDs with the domain `EventId` newtype
- [x] 1.6 Name the specialized mutation result after its single-entity-event invariant

## 2. GraphQL contract

- [x] 2.1 Add required `eventId` fields to Book and Author mutation payloads and populate all four create/update resolvers
- [x] 2.2 Add GraphQL unit coverage for payload values and out-of-scope payload stability
- [x] 2.3 Regenerate `schema.graphql` and verify the new fields are non-null IDs

## 3. Cross-layer verification

- [x] 3.1 Add Book create/update E2E coverage linking returned event IDs and event-set IDs to history and restore
- [x] 3.2 Add Author create/update E2E coverage linking returned event IDs and event-set IDs to history and restore
- [x] 3.3 Run mutation- and restore-related E2E tests when their database and authentication environment is available
- [x] 3.4 Name and structure create/update E2E tests around the mutation-to-history-to-restore contract

## 4. Documentation and validation

- [x] 4.1 Document operation-level and entity-event identifiers in `docs/architecture/event-recording.md`
- [x] 4.2 Add a changelog entry for the additive GraphQL fields if this repository's changelog convention requires it
- [x] 4.3 Run `cargo fmt --check`
- [x] 4.4 Run `cargo clippy --all-targets --locked -- -D warnings`
- [x] 4.5 Run `cargo test --locked`
