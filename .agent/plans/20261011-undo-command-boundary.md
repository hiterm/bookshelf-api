# Separate Undo execution from history reads

This living ExecPlan follows `.agent/PLANS.md` and covers the first stage of API
#367 R2. Maintain Progress, discoveries, decisions and outcomes as work proceeds.

## Purpose / Big Picture


Keep Undo behavior stable while giving its write workflow an explicit atomic
boundary. History readers and their mocks should no longer be able to execute
Undo. Mutation wiring must use a separate command interactor and a dedicated
PostgreSQL command. A user can still undo an owned operation, receive the new
operation ID, and observe fresh history or the existing conflict response.

## Progress


- [x] (2026-10-11) Inspect current main and existing Undo tests; create feature branch.
- [x] (2026-10-11) Milestone 1: Separate command interface, implementation and application wiring; all verification passed.
- [x] Milestone 1 plan updated.
- [x] (2026-10-11) Create draft PR #376 and record partial R2 status in Issue #367.

- [x] (2026-10-11) Review follow-up: document touched command functions; fmt, clippy and 190 unit tests passed.
- [x] Review follow-up plan updated.
- [ ] Confirm incremental CodeRabbit review and record its docstring check result.

## Surprises & Discoveries


The initial checkout predates PR #375; origin/main already contains it. Fetch
succeeded with escalated execution after the sandbox could not reach the proxy.
Undo remains implemented on HistoryQueryInteractor and HistoryRepository at
baseline b0aa2ae. Existing PostgreSQL Undo tests cover successful history, missing
referenced authors and multi-author rollback with name conflicts. Rust is absent from PATH;
install an isolated toolchain under /workspace/work rather than changing a
shared environment. Docker needs escalated access to its local socket.

## Decision Log


Decision (2026-10-11): implement R2's interface/boundary stage as one reviewable
change. Use a use-case-owned UndoCommand port because it describes a complete
application command, not a read repository. Keep all existing transaction SQL
and inverse-state helpers intact. Sharing those helpers with ordinary writes is
later work and must not silently reduce SERIALIZABLE isolation.

Decision (2026-10-11): keep is_operation_undoable on the read repository. Its
result is advisory, not permission to bypass lock-time revalidation. Preserve
GraphQL shape and errors. Existing E2E cases exercise the unchanged public
behavior; no new endpoint or duplicated E2E scenario is required.

## Outcomes & Retrospective


Implementation and verification are complete. Commit 6830cdb is published in
draft PR https://github.com/hiterm/bookshelf-api/pull/376. Issue #367 records R2
as in progress, with this first stage and remaining work explicitly separated.
Format and both clippy configurations passed. Unit tests: 190 passed.
PostgreSQL library tests: 252 passed and one existing ignored; integration tests:
5 atomic-save, 4 persistence-conflict and 1 decoding passed. API E2E: 53 passed.
Schema regeneration has no diff. Undo SQL execution and helpers match the
baseline exactly. This stage does
not finish all of R2: shared revision generation and query/execution condition
contract tests remain separate follow-up work.

## Context and Orientation


src/use_case/interactor/history.rs currently contains history reads plus the
Undo command. src/domain/repository/history_repository.rs exposes both reads
and writes. src/infrastructure/history_repository.rs delegates Undo to
src/infrastructure/undo.rs, where one SERIALIZABLE PostgreSQL transaction locks
and revalidates current state, applies inverse changes, records fresh immutable
snapshots (Revisions), and commits a new Operation with its entity changes.
Dependency injection in src/dependency_injection.rs currently uses the history
query interactor for both GraphQL queries and mutations.

## Plan of Work


Add src/use_case/port/undo.rs with UndoCommand, accepting UserId and OperationId
and returning the new OperationId or DomainError. Its documentation must require
lock-time validation, atomic current/history writes, rollback on every error and
return only after commit. Add HistoryCommandInteractor in the existing history
module to validate string inputs and map domain errors. Remove Undo from the
history read trait, interactor and repository implementation. Add PgUndoCommand
to the existing undo module and expose that module for dependency injection.
The command delegates to the same private SQL workflow. Wire AppMutation to a
new HC alias; loaders keep HQ. Route existing DB Undo tests through PgUndoCommand.
Add tests for invalid owner/ID and exact error categories, preserving the
existing owner/target and returned-operation test. Update current architecture
documents to explain the explicit command exception and advisory reads.

## Milestone 1


After the separation, a mock UndoCommand alone can exercise command validation,
owner/target forwarding, new operation identity and failures. Query mocks have
no write method. Run mandatory checks in order, then DB-feature tests and
existing API E2E. Confirm schema generation has no diff. Record results here
and commit the implementation with this plan.

## Concrete Steps


Work in /workspace/bookshelf-api on refactor/undo-command-boundary, based on
origin/main b0aa2ae. Install Rust only in /workspace/work/cargo and
/workspace/work/rustup if needed. Set CARGO_HOME, RUSTUP_HOME and extend PATH for
each verification command. Use a disposable PostgreSQL 15 container listening
only on loopback, with test user/database bookshelf and local test password.
These credentials are fixtures, not production secrets.

    cargo fmt
    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test --locked
    DATABASE_URL=postgres://bookshelf:password@localhost:5432/bookshelf cargo test --features test-with-database --locked

Start the existing bookshelf-jwks-server binary and API as specified by
.github/workflows/e2e.yml, using port 9999 and 8080 respectively and test JWT
settings. Run cargo test -p bookshelf-e2e --locked -- --test-threads=1.
Stop only task-owned processes and the disposable PostgreSQL container after
validation. Generate schema via the existing generator and verify no diff.

## Validation and Acceptance


All mandatory checks pass. Command unit tests reject invalid inputs without
calling the mock, forward the owner and target, return the committed Undo ID,
and preserve Conflict, NotFound, AuthorNameConflict and infrastructure failures.
Existing database tests must exercise PgUndoCommand and prove successful Undo,
stale-state rejection and complete rollback. Existing E2E must still return the
same GraphQL fields and error contract. SQL statements in the execution function
must be unchanged; schema.graphql must be unchanged.

## Idempotence and Recovery


Tests create isolated sqlx databases or unique E2E owners. Do not use production
DBs. Check for existing ports before starting services. If setup fails, fix or
report the specific blocker; never claim unrun tests passed. Keep logs outside
the repository in /workspace/work. Retry checks after meaningful fixes.

## Artifacts and Notes


Baseline: b0aa2ae (merge of PR #375). Logs live outside the repository under
/workspace/work/undo-*.log. The final verification script exited zero with
“All Undo validation checks passed”. Issue status must say R2 is partial.

## Interfaces and Dependencies


Use existing async_trait, mockall and sqlx. UndoCommand is Send + Sync + 'static
and has async undo_operation(&self, &UserId, &OperationId) returning
Result<OperationId, DomainError>. HistoryCommandInteractor implements the existing
HistoryCommandUseCase. PgUndoCommand owns a PgPool and implements UndoCommand.
No new dependencies, migration, endpoint or automatic retry.

Revision note (2026-10-11): initial plan records the bounded interface stage and
preserves database execution semantics to limit risk before deeper sharing.

Revision note (2026-10-11): completed implementation and validation. DB-feature
compilation found a test whose post-command read still used the old local
variable; explicitly use PgHistoryRepository for that read and PgUndoCommand for
the write. All checks passed after the correction.

Revision note (2026-10-11): created draft PR #376 and updated the tracking
Issue; task-owned test processes and PostgreSQL container have been stopped.

Revision note (2026-10-11): CodeRabbit reviewed e7d3d56 without actionable code
findings and assessed minimal merge risk. Its docstring coverage check warned
at 61.11% against an 80% threshold; add focused Rust documentation to the
command constructors, execution methods and input/error contract tests. Run
mandatory checks, push and request incremental review. No runtime change or new
E2E scenario is involved.

Revision note (2026-10-11): documentation-only Rust changes passed all mandatory
checks. An initial sandbox test run could not bind local JWKS HTTP sockets;
rerunning the same checks with socket access passed all 190 tests. Existing
CI, E2E, actionlint and zizmor passed on e7d3d56. Incremental review is pending.
