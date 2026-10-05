# Save books and new authors atomically

This living ExecPlan follows `.agent/PLANS.md`. Update Progress, discoveries, decisions and outcomes with each milestone.

## Purpose / Big Picture

Issue hiterm/bookshelf#404 concerns author creation partially succeeding before a book save. Add and edit must save the book, new authors, links and history together. On an author-name conflict, roll back everything, refresh authors in the browser, replace unchanged pending selections with exact-name existing authors, tell the user the book is not saved, and let them save again.

## Progress

- [x] (2026-10-05 JST) Inspect repositories and update from latest main; create codex/atomic-book-authors branches.
- [x] (2026-10-05 JST) Milestone 1: Implement backend contract and atomic persistence; verify unit and real database rollback tests.
  - [x] plan updated
- [ ] Milestone 2: Implement browser recovery, generated schema and mock contracts; verify UI and E2E behavior.
  - [ ] plan updated
- [ ] Milestone 3: Complete required checks and publish mutually linked PRs referencing issue 404.
  - [ ] plan updated

## Surprises & Discoveries

The repositories are symlinks outside the virtual workspace, so writes and Git operations need sandbox escalation. Existing import can reuse authors, but this feature must reject existing names. Use the ordinary author repository create method instead. Operation history already supports multiple entity changes and Undo.

## Decision Log

2026-10-05, user-approved: extend existing inputs with `newAuthorNames: [String!]! = []`; keep authorIds. Deduplicate exact names and IDs, use empty yomi for new authors and deterministic name insertion order. Keep strict conflict semantics in the API; recover only the browser selection after rollback. Do not automatically resubmit or guarantee idempotency after response loss. Use one CreateBook/UpdateBook operation and return the book revision. No database migration is required.

## Outcomes & Retrospective

Backend implementation is complete: 179 unit tests, five dedicated real-DB atomic-save tests, and 52 HTTP E2E tests passed. Earlier full database-feature validation passed 236 library tests plus the five new tests. Frontend unit/type/mock/demo checks have passed; real-API UI validation and PR publication are pending. Do not merge or deploy these PRs as part of this work.

## Context and Orientation

In bookshelf-api, src/presentation/graphql/object.rs defines inputs, src/use_case/dto/book.rs carries them to src/use_case/interactor/book.rs, which owns transactions. AuthorRepository.create records author history in its caller's transaction. Domain, use-case and presentation errors must carry a typed author-name conflict to GraphQL extensions code CONFLICT and reason AUTHOR_NAME_CONFLICT. In bookshelf, src/features/books/AddBookButton.tsx and BookEdit.tsx currently create authors before saving. Replace that workflow with one book mutation. src/mocks/mockStore.ts and handlers.ts supply demo and test behavior and must enforce the same all-or-nothing contract.

## Plan of Work

Milestone 1 extends both book input DTOs and GraphQL inputs, validates ownership before writes, creates authors in the book transaction and preserves response metadata. Add typed error mapping and database tests proving rollback after an earlier author insert and after book persistence failure. Check duplicate input, concurrent conflict, tenant separation, old clients, history and Undo. Run the backend checks below and update this plan before committing.

Milestone 2 generates the frontend schema/types from the modified backend using GRAPHQL_SCHEMA_PATH, splits selected authors into existing IDs and new names, and removes resolvePendingAuthors. A shared recovery hook fetches fresh authors on the typed conflict, matches exact names only against unchanged submitted pending selections, deduplicates resolved IDs, and notifies without saving. Hold the submission guard until recovery finishes. Preserve edits/removals made during the request. Failed refresh or no match keeps input and reports an error. Invalidate author and book caches on success. Update demo/mock behavior and component/E2E regressions, then update this plan before committing.

Milestone 3 runs all required checks, documents the new contract, and publishes two PRs. API body contains `Refs hiterm/bookshelf#404`, frontend body `Closes #404`; cross-link both and state API-first release dependency. Attach both PRs to this chat. The frontend PR must not merge before its schema dependency is available.

## Concrete Steps

From bookshelf-api run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, and `cargo run --bin gen_schema`. Database tests require a local PostgreSQL DATABASE_URL with permission to create test databases. Use README.md's JWT/JWKS server setup for `cargo test -p bookshelf-e2e -- --test-threads=1`.

From bookshelf run `GRAPHQL_SCHEMA_PATH=../bookshelf-api/schema.graphql pnpm run generate`, `pnpm run lint:fix`, `pnpm run format`, `pnpm run test`, `pnpm run typecheck`. Run relevant `pnpm run test:e2e:mock-api`, `pnpm run test:e2e:demo-mode`, and `pnpm run test:e2e:integration` suites with local resource overrides if needed. Do not change shared concurrency configuration. Record exact successful commands and counts below.

## Validation and Acceptance

An API request with new authors A and B, where B already exists, returns the typed conflict and leaves no A, book mutation or failed-operation history. A subsequent browser refresh replaces B's unchanged pending selection with its real ID and reports that the book remains unsaved. Clicking save creates A and saves the book once. Test this for create and update. A book write failure after author insertion leaves no new author. One operation contains all successful changes, and immediate Undo reverts them. Invalid/cross-user IDs fail; another user's same author name does not conflict. Concurrent same-user names cannot both be created. Old clients omitting newAuthorNames still work.

## Idempotence and Recovery

No migration or destructive data cleanup is needed. Test users/databases are isolated. Keep input after errors and do not automatically retry writes. A committed response lost in transit is outside this feature's guarantee. Preserve user changes and retry failed tooling only after diagnosing the cause.

## Artifacts and Notes

Initial main revisions: frontend fdd6784; API 232ae40. Verification: cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked (179), DATABASE_URL=postgres://postgres:password@localhost:55404/postgres cargo test --locked --features test-with-database --test atomic_books (5), and TEST_SERVER_URL=http://localhost:8080 cargo test --locked -p bookshelf-e2e -- --test-threads=1 (52) passed. The dedicated test container is bookshelf-404-postgres. PR URLs will be recorded after publication.

## Interfaces and Dependencies

Use existing Rust transaction/repository traits and React Query hooks; add no dependencies. CreateBookInput and UpdateBookInput gain newAuthorNames with an empty default; corresponding Rust DTOs gain Vec<String>. GraphQL author-name conflicts have code CONFLICT and reason AUTHOR_NAME_CONFLICT. Recovery changes form references only, never stored author attributes. Backend deploy precedes the frontend; local type generation uses GRAPHQL_SCHEMA_PATH until the backend release exists.

Revision note: initialized from the approved design on 2026-10-05.

Revision note: completed backend atomic-save milestone and recorded verification evidence on 2026-10-05. Canonical UUID strings are deduplicated by identity, including case variants.
