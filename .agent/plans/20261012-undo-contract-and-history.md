# Fix the Undo contract in tests and share history primitives

This living ExecPlan follows `.agent/PLANS.md`. Update Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective in every milestone commit.

## Purpose / Big Picture


Undo must reject stale operations without changing any current or historical rows. The history screen's eligibility flag is an advisory check of revision identity, not a guarantee against dependencies or concurrent writes. First pin that distinction in database tests. Then remove duplicated revision allocation and operation-change recording from ordinary mutations and Undo, and share restoration snapshot recording without changing transaction ownership, timestamps, tenant boundaries, or public GraphQL behavior. Demonstrate both through real PostgreSQL tests and existing API end-to-end tests.

## Progress


- [x] (2026-10-12) Milestone 1: Add and validate Undo contract tests; prepare the PR against main.
  - [x] plan updated
- [x] (2026-10-12) Milestone 2: Share restoration/history primitives and validate; prepare the dependent PR.
  - [x] plan updated
- [ ] Milestone 3: Address CodeRabbit review and CI, approve when sound, and stop before merge.
  - [ ] plan updated

## Surprises & Discoveries


Eligibility checks only target ownership/type, nonempty changes, current existence, and latest revision equality. Missing source authors and restored author-name collisions are intentionally detected when executing Undo. PostgreSQL foreign keys keep source revision references present; shared helpers nevertheless reject a zero-row INSERT SELECT rather than silently write misleading history. Existing `tests/persistence_conflicts.rs` already proves rollback after partial inverse writes and corrupt persisted names; new contract tests complement these rather than replacing them.

## Decision Log


Decision: Preserve the advisory contract rather than expand the query into an execution simulation. Rationale: state can change between query and command, so authoritative revalidation must remain inside the SERIALIZABLE transaction. Date/author: 2026-10-12, Codex.

Decision: Record fresh snapshots from owned current rows after restoration and use the same source-to-current SQL in ordinary restore and Undo. Rationale: both paths preserve exactly the persisted fields and links, while ordinary callers keep decoding and their supplied update timestamp. The enum parsers only accept exact canonical strings, so copying validated source enums does not alter ordinary normalization. Allocation now uniformly uses checked integer addition, making Undo overflow an internal error with full rollback. Date/author: 2026-10-12, Codex.

Decision: Use separate stacked PRs. Rationale: tests can be reviewed independently before the refactor; neither PR needs merging to review the second. Date/author: 2026-10-12, Codex.

## Outcomes & Retrospective


Milestone 1 is implemented: five PostgreSQL contract tests cover both entities and real lock contention. Required fmt/clippy/unit checks pass (190 unit tests); database library tests pass (252 plus one existing ignored), with 15 integration tests passing. Existing API E2E passes all 53 tests and generated GraphQL schema is unchanged. PR #377 targets main; its reviewed restoration-field nitpick is fixed in `1073bbd`. Milestone 2 now shares source-to-current restoration, persisted snapshot recording, checked revision allocation, and change inserts. The SERIALIZABLE command orchestration is unchanged. Three new PostgreSQL round-trip/overflow tests pass, covering update/resurrection, populated/empty links, nullable purchase dates, all payload fields, source creation times, ordinary returned timestamps, Undo transaction timestamps, owner-scoped allocation, and rollback after an earlier inverse. Complete validation passes: 191 unit tests, 253 database library tests (one existing ignored), 18 integration tests, and 53 API E2E tests. Generated schema is unchanged. The existing command extraction was merged as PR #376; this plan covers Issue #367 R2's remaining work. No schema migration or new endpoint is intended. Stop before merging both PRs.

## Context and Orientation


`src/infrastructure/history_repository.rs` implements the read-only eligibility query. `src/infrastructure/undo.rs` owns the complete command: begin SERIALIZABLE transaction, lock the owned operation, lock all entity advisory keys in sorted order, lock current rows, compare after revisions, restore authors before books, delete books before authors, record a new inverse operation, commit. A revision is an immutable numbered entity snapshot; a change links an operation to its before/after revisions, using NULL for absence.

Ordinary mutations use `PgTransaction` in `src/infrastructure/transaction.rs`, with immutable owner and operation context. `src/infrastructure/history_recording.rs` allocates revisions and records snapshots/changes; book and author repositories restore old snapshots before calling it. Keep domain decoding and repository-specific failure categories in callers. Shared current-row restoration lives in `src/infrastructure/revision_restoration.rs`; shared snapshot recording reads the just-persisted owned entity and author links in the same transaction. Shared primitives take a PostgreSQL connection plus explicit owned context and never commit or begin transactions. Every query must scope by owner as well as UUID. Preserve ordinary restore's application timestamp and Undo's transaction timestamp.

## Plan of Work and Milestones


Milestone 1 adds `tests/undo_contract.rs`, gated by `test-with-database`. Seed isolated books and authors with owned operations/revisions. Cover create/update/delete, stale or absent current entities, empty/baseline/foreign operations, other-tenant independence, stale advice, dependency/name conflicts, and concurrent commands. Assert exact error classes and unchanged snapshots of all nine current/history tables on rejection. Concurrent tests must wait for observable database lock contention before releasing a blocker, with a bounded timeout, rather than guess scheduling with sleeps. Run the required checks, database suite, and existing E2E suite, then commit tests and this updated plan on `test/undo-eligibility-contract`. Create a PR against main and request CodeRabbit review.

Milestone 2 creates a branch from milestone 1. Consolidate owner-scoped revision allocation with checked overflow, inverse/ordinary operation-change inserts, and restoration snapshot recording in the infrastructure history module. Prefer typed helpers over table-name interpolation and preserve existing persisted-value error handling. If sharing current-row restore SQL changes canonical enum decoding or timestamps, retain caller-specific validation/current-row writes and share only the safe persistence steps; document the decision. Add meaningful tests comparing restored entity fields, lifecycle times, relationships, revision links, and another tenant. Run the same validations and commit code/tests/updated plan. Create a PR whose base is milestone 1's branch and request CodeRabbit review.

Milestone 3 inspects each PR's latest diff and CI at its exact head, addresses actionable review findings with required checks before each code commit, and requests approval only after own review is clear. If external review quota prevents completion, distinguish that limitation from local/CI validation. Update Issue #367 with accurate PR/status links. Leave both PRs unmerged and report their dependency order.

## Concrete Steps


Work in `/workspace/bookshelf-api`. Use Rust pinned by `rust-toolchain.toml`; in this workspace export `CARGO_HOME=/workspace/work/cargo`, `RUSTUP_HOME=/workspace/work/rustup`, and prepend `/workspace/work/cargo/bin` to PATH. Use a disposable PostgreSQL 15 database, `DATABASE_URL=postgres://bookshelf:password@localhost:5432/bookshelf`. SQLx creates/migrates a separate database per test; never run fixtures on a production database.

Before every code commit run, in this order:

    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test --locked

Then run:

    cargo clippy --features test-with-database --all-targets --locked -- -D warnings
    cargo test --features test-with-database --locked
    cargo run --locked --bin gen_schema
    git diff --exit-code -- schema.graphql

Build `cargo build --locked -p bookshelf-api -p bookshelf-e2e --bins`; start the JWKS test server on 9999 and API on 8080 with PORT=8080, ALLOWED_ORIGINS=http://localhost:8080, JWT_AUDIENCE=test-audience, JWT_DOMAIN=test-issuer.local, JWKS_URL=http://localhost:9999/.well-known/jwks.json and DATABASE_URL above. Wait for health/JWKS readiness with a bounded timeout. Run `TEST_SERVER_URL=http://localhost:8080 cargo test -p bookshelf-e2e --locked -- --test-threads=1`; stop only task-owned processes afterwards.

## Validation and Acceptance


All commands exit zero; generated schema remains unchanged. Every rejected command leaves all current/history rows unchanged. Eligible create/update/delete cases produce exactly one inverse operation with correct before/after links. A newer same-owner revision invalidates earlier advice, while other-owner revisions do not. A dependency/name failure can follow true advice but still leaves no writes. Two contending commands produce one committed inverse, and the loser is stale Conflict or PostgreSQL serialization rejection. Existing E2E tests demonstrate unchanged API behavior. Final PR heads must have passing required CI checks and accurately reported review status.

## Idempotence and Recovery


Tests are isolated and repeatable. Restart the disposable database if needed without modifying persistent external data. On a test failure inspect logs under `/workspace/work`, fix the code or fixture, and rerun affected tests then required precommit checks. Do not weaken transaction isolation to accommodate concurrency tests. Keep feature branches intact; never edit or commit main. Do not merge, delete source branches, or reset away uncommitted work.

## Artifacts and Notes


PR #377 review follow-up: CodeRabbit approved with a valid nitpick that equal fixture attributes hid incorrect restoration. Success cases now use distinct before attributes and compare restored book/author fields and lifecycle start against the source. Added function documentation to address its coverage warning.

Milestone 1 evidence: `cargo test --features test-with-database --test undo_contract` reports `5 passed; 0 failed`; complete validation reports `All Undo validation checks passed`. Scratch logs use `/workspace/work/undo-contract-*.log`. Milestone 2 logs use `/workspace/work/shared-history-*.log` and finish with `All Undo validation checks passed`. CodeRabbit reviewed and approved the original #377 head, but re-review of the nitpick/docstring follow-up hit the included one-review-per-hour quota; that external limitation remains explicitly distinguished from local validation. Scratch logs and service scripts remain outside the repository in `/workspace/work`.

## Interfaces and Dependencies


No new dependency, public endpoint, or database schema is required. `HistoryRepository::is_operation_undoable` stays read-only. `UndoCommand::undo_operation` continues accepting validated `UserId` and `OperationId`, and returns a new ID only after commit. Shared infrastructure functions must accept caller-owned connections; operation ownership and isolation remain responsibilities of `PgTransaction` and `PgUndoCommand` respectively.
