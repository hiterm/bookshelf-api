# Normalize persistence error classification (Issue #367 R3)

This living ExecPlan follows `.agent/PLANS.md`. Maintain Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective at every milestone. The user authorized implementation, PR creation, and CodeRabbit review and responses after approving this plan on 2026-10-11 (Asia/Tokyo). E2E additions were explicitly approved earlier in this conversation.

## Purpose / Big Picture


Users should receive the same actionable error when an author name is already used, whether they create, update, restore, or undo an author. Today restore reports invalid input and Undo can report an internal failure for that same conflict. Conversely, invalid values already stored in the database must be reported as an internal failure, rather than asking users to correct otherwise valid input. Demonstrate the change through real HTTP requests, checking GraphQL error codes and proving failed writes leave both current data and history unchanged.

## Progress


- [x] (2026-10-11 JST) Inspect repository guidance, current persistence paths, error contracts, and frontend error consumers; create `codex/persistence-error-contract` from fetched `origin/main`.
- [x] (2026-10-11 JST) Planning milestone: record classification, compatibility, implementation steps, and verification requirements; stop before implementation.
  - [x] plan updated
- [x] (2026-10-11 JST) Milestone 1: Implement persisted-value decoding with unit and database regressions; update the active error specification.
  - [x] plan updated
- [x] (2026-10-11 JST) Milestone 2: Share author-name constraint classification across writes, restore, and Undo; prove conflict rollback and HTTP behavior.
  - [x] plan updated
- [ ] Milestone 3: Complete reviewable delivery and CodeRabbit review/response.
  - [ ] plan updated

## Surprises & Discoveries


The baseline inspected is API commit `5a123a1909f97f43a8e67ea2daaacfc784468026`, package version 2.17.6. `DomainError::AuthorNameConflict` already propagates through `UseCaseError` and `PresentationalError` to GraphQL `code = CONFLICT` and `reason = AUTHOR_NAME_CONFLICT`. No new public error type is necessary.

`src/infrastructure/author_repository.rs` currently recognizes SQLSTATE 23505 (PostgreSQL unique constraint violation) and the exact constraint `author_user_id_name_unique` for create/update. Restore has a precheck and a separate constraint handler, both returning Validation. `src/infrastructure/undo.rs::restore_author` lets the same SQL error reach the generic InfrastructureError conversion. Neither path was reproduced during planning; these are static findings.

Current Book reconstruction uses input-validation conversions for stored fields. History and backup already classify format/store decode failures as Unexpected, but duplicate this logic. Other validated stored fields, such as title, priority, author name, user ID, and revision numbers, can still flow through Validation in reconstruction. The plan covers these conversion boundaries without inventing stricter rules for historical data.

The frontend inspected uses the typed name-conflict reason in `src/features/books/useAuthorConflictRecovery.ts` for book-save recovery. Its generic `src/components/errors/appError.ts::normalizeError` displays safe GraphQL messages and retains sanitized extensions. Searches found restoreAuthor in mock handlers/store, but did not establish a dedicated live restore/Undo UI consumer. This is a limited static compatibility review, not proof about every external client. Recheck the frontend revision before implementation.

The shell invocation used during inspection could not find rustc on PATH. Docker listed no running containers. Toolchain readiness, DB readiness, and successful tests are unverified. A normal git fetch failed to connect to the session proxy; the required retry with elevated sandbox permissions succeeded. Do not report a test pass or authentication failure based on these observations.

## Decision Log


Decision: Unify recognized author-name conflicts using the existing AuthorNameConflict variant, including restore and Undo. Rationale: conflicts depend on current state rather than malformed input, and the existing typed reason provides consistency without schema changes. Public restore behavior changes from VALIDATION_ERROR to CONFLICT; recognized Undo collisions change from INTERNAL_ERROR to CONFLICT. Author names retain their existing actionable message. Date/Author: 2026-10-11, Codex.

Decision: Place context-specific SQL classification and persisted enum decoding in `src/infrastructure/error.rs`. Rationale: persistence semantics belong in infrastructure; changing global domain From implementations would also alter legitimate input validation. Unknown SQL errors retain their original cause and InfrastructureError classification. Date/Author: 2026-10-11, Codex.

Decision: Convert only constructor failures arising from stored data into Unexpected, with field/context information, and preserve errors already classified as internal. Rationale: this distinguishes storage inconsistency from input mistakes without silently accepting bad data or weakening validation. Do not log in these helpers; presentation already records internal failures once. Date/Author: 2026-10-11, Codex.

Decision: Add HTTP E2E tests as well as mandatory unit and real-DB tests. Rationale: the public code changes need verification through a valid resolver document and the real application stack. The user explicitly chose E2E additions. Date/Author: 2026-10-11, user and Codex.

Historical decision: Stop after saving this plan. Rationale: the user's latest instruction narrows the authorized work to planning. Do not change source, run implementation tests, commit, publish a PR, or update Issue status in this planning turn. Date/Author: 2026-10-11, user and Codex.

## Outcomes & Retrospective


Implementation and local verification are complete. The typed author-name conflict is shared across all four write paths; stored-data validation is separated from input validation. Mandatory fmt/clippy/tests passed (188 unit tests). The database-feature suite passed 250 library tests with one pre-existing ignored test, five atomic-save tests, three new conflict/rollback tests, and one new persisted-read test. Feature-enabled clippy passed. HTTP targeted tests passed nine cases and full API E2E passed 53 cases. Regenerating schema.graphql produced no diff. Four decoding regressions and three conflict regressions failed on the original baseline as expected. PR publication and CodeRabbit review/response remain pending.

## Context and Orientation


Work in `/workspace/bookshelf-api`. Domain errors are defined in `src/domain/error.rs`; `src/use_case/error.rs` and `src/presentation/error.rs` translate them for clients. A GraphQL error has a public message, an `extensions.code`, and a resolver path identifying the failing field. Internal errors use `INTERNAL_ERROR` and the fixed message `Internal server error`; existing tracing records diagnostic causes on the server.

`src/infrastructure/book_repository.rs::book_from_row` constructs current Books from database rows. `src/infrastructure/history_repository.rs` converts Operation, BookRevision, AuthorRevision, and change rows. `src/infrastructure/backup.rs` converts current and historical enum strings into export projections. Author read paths and restore reconstruct names in `src/infrastructure/author_repository.rs`. A tenant is the authenticated owner represented by user_id; retain owner-scoped queries and composite identities.

Use cases open a transaction before mutations and commit only after current-state and history writes succeed. A transaction groups writes so a failure discards all of them. History consists of Operation rows describing actions, immutable Revision snapshots, and OperationChange rows linking before/after revisions. Undo owns a separate SERIALIZABLE transaction, locks and revalidates affected entities, and writes inverse state plus fresh history. Keep these boundaries, lock order, and execution-time checks unchanged.

## Plan of Work


### Milestone 1: Decode persisted values consistently


In `src/infrastructure/error.rs`, introduce internal helpers `persisted_book_format(&str) -> Result<BookFormat, DomainError>`, `persisted_book_store(&str) -> Result<BookStore, DomainError>`, and `persisted_value<T>(&'static str, Result<T, DomainError>) -> Result<T, DomainError>`. The generic helper converts Validation from a stored-value constructor into Unexpected with context; it leaves other error variants unchanged. These helpers reuse existing parsing rules and add no dependency or domain validation rule.

Replace duplicated format/store parsing in current Book, BookRevision, and backup conversions. Wrap validated constructors fed by stored fields in current Book/Author reads, Author restore, Operation/Revision conversions, and optional revision numbers on change rows. Inspect history_recording and transaction metadata reads for analogous stored-number validation; cover any found conversion, while preserving overflow's existing internal classification. Do not wrap user-supplied DTO validation or convert SQL errors into input/conflict errors globally.

Add ordinary unit tests, enabled without the database feature, for every legal enum value and invalid values. Add row-conversion tests with one invalid field at a time so a bad title/ISBN/priority/name/revision cannot be mistaken for bad format. Valid rows must reconstruct unchanged. Add a regression proving an invalid input still reports Validation. Unit tests must demonstrate failure on the baseline rather than merely call the new helper in isolation.

Add real database read tests for current Book, history, and backup. A disposable sqlx test database can insert a synthetic format/store into the existing lookup table and reference it from otherwise valid rows; this exercises unrecognized application enum values without dropping constraints. Verify internal classification through all relevant readers. Do not inject corruption into the shared E2E database or any production database. Update only `openspec/specs/api-error-contract/spec.md`, adding the input-versus-storage distinction and safe public result; preserve archived artifacts.

Acceptance is that legal stored values retain results, corrupt stored fields yield an internal error, and genuine invalid client input retains VALIDATION_ERROR. Run the ordinary unit tests and database-feature suite below before marking this milestone complete.

### Milestone 2: Classify author name conflicts consistently


Move `classify_author_name_write_error(sqlx::Error, &str) -> DomainError` from the author repository to `src/infrastructure/error.rs` with crate-internal visibility. Match both SQLSTATE 23505 and the exact named author constraint. Create/update retain their current behavior. Restore's precheck returns AuthorNameConflict and its authoritative write uses the shared classifier to handle collisions occurring after the precheck.

In Undo's `restore_author`, read the source revision name using the same transaction, user_id, author_id, and source revision. Obtain the name before the write; a PostgreSQL transaction that has failed cannot be queried for the name afterward. Map only the current-author restoration write's error through the shared classifier. Keep revision/history writes on the generic infrastructure path, so unrelated uniqueness or foreign-key failures do not become name conflicts. A missing source revision must fail as an internal invariant error rather than successfully restore zero rows.

Extend the existing restore name-conflict database test to assert AuthorNameConflict and compare complete owner-scoped rows before the attempted action and after rollback. Add an Undo test where an author's old name was claimed by another author after rename/delete; the attempt must report the typed conflict, preserve both authors, and create no committed inverse Operation or Revision. Include a multi-entity Undo case with a deterministic later collision after an earlier inverse write, proving rollback of the earlier write and history too. Reuse existing fixtures where they match these conditions.

Test the real PostgreSQL unique violation directly, independent of restore's precheck, so the write-error branch is proven. Test a different unique constraint, a foreign-key error, and a non-database sqlx error; all remain InfrastructureError. Keep same-name authors in different tenants legal. Do not infer conflicts from SQLSTATE alone and do not add global retry behavior.

Add resolver tests in `src/presentation/graphql/schema.rs` for restoreAuthor and undoOperation, using mocks whose required calls prove resolver execution. Check message, CONFLICT, AUTHOR_NAME_CONFLICT, and field path. Pass an error produced by the persistence decoder through an actual schema query and assert INTERNAL_ERROR, the fixed message, and absence of a synthetic private marker. Existing generic conversion tests remain useful but do not replace these specific regressions.

Add HTTP tests to `e2e/tests/graphql_restore.rs` and `e2e/tests/graphql_undo.rs`. Create A with name Original, rename A, create B with Original, then attempt A's restore or Undo of its rename. Require a non-empty errors array, expected code/reason/path, and unchanged state/history. Remove or rename the conflicting author and retry the same action as a successful control. Check every setup mutation for errors. Update the active error specification with both conflict scenarios and document the code changes in the eventual PR.

Acceptance is identical actionable classification on all four write paths, no partial commit, successful non-conflicting controls, and internal classification for unknown DB failures.

### Milestone 3: Verify and deliver


Resolve the Rust PATH/toolchain setup first; inspect available tools without printing credentials. Use the pinned toolchain in `rust-toolchain.toml` and the lockfile. Recheck actual runtime readiness; do not reuse historical test counts as current evidence. A feature branch is already present; inspect status before changing or committing files.

Before each code commit, run the three mandatory checks below in order and fix any failures. Run the database-feature suite, changed HTTP tests, and full API E2E suite. Review whether clippy with the DB feature is needed for newly feature-gated test code. Review the frontend error consumers again against the current checkout. No GraphQL schema shape changes are intended; check that schema.graphql stays unchanged. Any additional frontend code changes require a concrete compatibility finding and a recorded plan update.

When implementation resumes, deliver independent milestone commits with this plan updated in each one, then create a reviewable PR referencing Issue #367 R3. Update only R3's status and PR link in the Issue's tracking table after confirming the PR exists, preserving the investigation text and other items. Do not close #367, which tracks other work. Merging and deployment are outside the planned delivery.

## Concrete Steps


All Rust commands run from `/workspace/bookshelf-api`. The required pre-commit sequence is:

    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test --locked

Tests must exit zero. Record the actual counts and ignored tests when executed; no counts are predicted here. For database tests, set DATABASE_URL for an isolated local PostgreSQL user permitted to create sqlx test databases, then run:

    cargo test --locked --features test-with-database
    cargo clippy --all-targets --locked --features test-with-database -- -D warnings

Provision a task-owned PostgreSQL container from the pinned image in docker-compose-test.yml on an unused local port. Follow the managed environment's Docker proxy and certificate guidance. Create an isolated E2E database with that container's psql and set DATABASE_URL to it. Keep local test credentials out of committed configuration. Save scratch logs under `/workspace/work/r3/`, not tracked source directories.

Start JWKS and application processes separately, retaining their PIDs and logs:

    cargo run --locked -p bookshelf-e2e --bin bookshelf-jwks-server
    PORT=8080 ALLOWED_ORIGINS=http://localhost:8080 JWT_AUDIENCE=test-audience JWT_DOMAIN=test-issuer.local JWKS_URL=http://localhost:9999/.well-known/jwks.json cargo run --locked

DATABASE_URL must be set for the application. The JWKS endpoint is http://localhost:9999/.well-known/jwks.json; the application readiness endpoint is http://localhost:8080/health. Use an available alternative application port if occupied and propagate it to ALLOWED_ORIGINS and TEST_SERVER_URL. Check readiness and process exits within a bounded wait, then run:

    TEST_SERVER_URL=http://localhost:8080 cargo test --locked -p bookshelf-e2e --test graphql_restore --test graphql_undo -- --test-threads=1
    TEST_SERVER_URL=http://localhost:8080 cargo test --locked -p bookshelf-e2e -- --test-threads=1

Do not start the second run until the targeted tests pass. Preserve only relevant sanitized diagnostics. Stop task-owned servers and containers when done. Before publishing, inspect `git --no-pager diff --check`, the full diff, status, and changed schema/spec files. Use exact PR body text through a structured tool or body file.

## Validation and Acceptance


The externally visible restore collision must contain a safe author-name message and extensions `{"code":"CONFLICT","reason":"AUTHOR_NAME_CONFLICT"}` at path `["restoreAuthor"]`; Undo must yield the same extensions at `["undoOperation"]`. Inputs such as revisionNumber zero still return VALIDATION_ERROR. Stored enum/validated-field failures yield INTERNAL_ERROR and `Internal server error`, without the stored synthetic marker, SQL text, constraint details, or tenant ID in the client response.

For rollback verification, compare deterministic full-row snapshots of owner-scoped author, book, book_author, operation, author_revision, book_revision, book_revision_author, author_operation_change, and book_operation_change as appropriate. Counts alone cannot prove unchanged data. The failed operation must leave no newly committed rows; a subsequent non-conflicting retry must succeed and append the expected fresh revision once. Scope snapshots to the fixture owner and sort rows before comparison.

Show that the new row-decoding and restore/Undo conflict regressions fail on the baseline and pass after implementation. Unknown SQL failures must retain infrastructure classification and their diagnostic cause. Preserve all existing successful operation, tenant separation, and history tests. Passing tests are required evidence; compilation alone does not complete R3.

## Idempotence and Recovery


No migration, data rewrite, stricter validation rule, or transaction isolation change is planned. SQL fault/corruption fixtures belong only to disposable test databases. Do not change shared databases, disable foreign keys globally, or clear unrelated containers. If tests fail, retain the fixture and sanitized logs long enough to diagnose, then rerun the affected checks after fixing the cause. Avoid automatic mutation retries; clients must deliberately resolve a conflict and resubmit. Reverting implementation commits restores the previous public classification because no stored schema/data conversion is introduced.

## Artifacts and Notes


Plan location: `.agent/plans/20261011-persistence-error-contract.md`. Baseline: `5a123a1909f97f43a8e67ea2daaacfc784468026`. Branch: `codex/persistence-error-contract`. Implementation evidence is recorded in Outcomes & Retrospective and scratch logs under `/workspace/work/r3/`. Append exact validation results, PR URL, and known limitations during future milestones. The review copy supplied to the user is a snapshot; the repository file is authoritative when implementation resumes.

## Interfaces and Dependencies


Use existing sqlx, anyhow, DomainError, and the existing presentation conversion. Helper visibility is `pub(crate)` or narrower as callers permit. Keep global `From<sqlx::Error> for DomainError` as InfrastructureError and domain parsing From conversions as Validation for input paths. Keep AuthorNameConflict's propagation and presentation reason unchanged. Use Rust unit tests, sqlx::test for isolated DBs, and existing bookshelf-e2e helpers for HTTP tests. Add no production dependency and no new endpoint.

Revision note (2026-10-11 JST): initialized after the user narrowed work to planning only; recorded existing typed conflict support, intentional public code changes, compatibility limits, and unverified runtime prerequisites.

Revision note (2026-10-11 JST): implementation, PR creation, and CodeRabbit review/response are now authorized; the earlier planning-only pause no longer applies.

Revision note (2026-10-11 JST): milestone 1 implements current/history/restore/backup decoding and owner/revision reconstruction. fmt, clippy, 187 unit tests, and the complete database-feature suite passed; separate baseline regression verification remains running. User repository owner reconstruction also uses the shared decoder.

Revision note (2026-10-11 JST): milestone 2 completes author-name constraint classification, actual post-precheck constraint injection, multi-author Undo rollback, resolver contracts, and combined restore/Undo HTTP conflict/recovery. A combined E2E in graphql_restore.rs exercises both endpoints instead of duplicating fixtures across two files. SQL classifier tests use a separate schema because named unique constraints create schema-scoped indexes. All verification counts above are actual current results; baseline regressions were independently demonstrated.
